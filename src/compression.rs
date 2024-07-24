use std::{collections::HashMap, io::{Cursor, Read, Write}};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

use crate::binio::{BitReader, BitWriter};

/// The size of compressed data in each chunk
#[derive(Debug, Clone, Copy)]
pub struct ChunkInfo {
    /// The size of the data when compressed
    pub size_compressed: usize,

    /// The size of the original uncompressed data
    pub size_raw: usize,
}

/// A CZ# file's information about compression chunks
#[derive(Default, Debug, Clone)]
pub struct CompressionInfo {
    /// Number of compression chunks
    pub chunk_count: usize,

    /// The compression chunk information
    pub chunks: Vec<ChunkInfo>,
}

impl CompressionInfo {
    pub fn write_into<T: WriteBytesExt + Write>(
        &self,
        output: &mut T,
    ) -> Result<(), std::io::Error> {
        output.write_u32::<LE>(self.chunk_count as u32)?;

        for chunk in &self.chunks {
            output.write_u32::<LE>(chunk.size_compressed as u32)?;
            output.write_u32::<LE>(chunk.size_raw as u32)?;
        }

        Ok(())
    }
}

pub fn compress(data: &[u8]) -> (Vec<u8>, CompressionInfo) {
    let mut part_data;

    let mut offset = 0;
    let mut count;
    let mut last: Vec<u8> = Vec::new();

    let mut output_buf: Vec<u8> = Vec::new();
    let mut output_info = CompressionInfo {
        ..Default::default()
    };

    loop {
        (count, part_data, last) = compress_lzw(&data[offset..], last);
        if count == 0 {
            break;
        }
        offset += count;

        output_buf.write_all(&part_data).unwrap();

        output_info.chunks.push(ChunkInfo {
            size_compressed: part_data.len(),
            size_raw: count,
        });

        output_info.chunk_count += 1;
    }

    if output_info.chunk_count == 0 {
        panic!("No chunks compressed!")
    }

    (output_buf, output_info)
}

fn compress_lzw(data: &[u8], last: Vec<u8>) -> (usize, Vec<u8>, Vec<u8>) {
    let mut count = 0;
    let mut dictionary = HashMap::new();
    for i in 0..=255 {
        dictionary.insert(vec![i], i as u64);
    }
    let mut dictionary_count = (dictionary.len() + 1) as u64;

    let mut element = Vec::new();
    if last.is_empty() {
        element = last
    }

    let mut output_buf = Vec::new();
    let mut bit_io = BitWriter::new(&mut output_buf);
    let write_bit = |bit_io: &mut BitWriter<Vec<u8>>, code: u64| {
        if code > 0x7FFF {
            bit_io.write_bit(1, 1);
            bit_io.write_bit(code, 18);
        } else {
            bit_io.write_bit(0, 1);
            bit_io.write_bit(code, 15);
        }
    };

    for c in data.iter() {
        let mut entry = element.clone();
        entry.push(*c);

        if dictionary.contains_key(&entry) {
            element = entry
        } else {
            write_bit(&mut bit_io, *dictionary.get(&element).unwrap());
            dictionary.insert(entry, dictionary_count);
            element = vec![*c];
            dictionary_count += 1;
        }

        count += 1;

        if dictionary_count >= 0x3FFFE {
            count -= 1;
            break
        }
    }

    let last_element = element;
    if bit_io.byte_size() == 0 {
        if !last_element.is_empty() {
            for c in last_element {
                write_bit(&mut bit_io, *dictionary.get(&vec![c]).unwrap());
            }
        }
        drop(bit_io);
        return (count, output_buf, Vec::new());
    } else if dictionary_count < 0x3FFFE {
        if !last_element.is_empty() {
            write_bit(&mut bit_io, *dictionary.get(&last_element).unwrap());
        }
        drop(bit_io);
        return (count, output_buf, Vec::new());
    }

    drop(bit_io);
    (count, output_buf, last_element)
}

pub fn decompress<T: ReadBytesExt + Read>(
    input: &mut T,
    chunk_info: &CompressionInfo,
) -> Vec<u8> {
    let mut output_buf: Vec<u8> = vec![];

    for block in &chunk_info.chunks {
        let mut buffer = vec![0u8; block.size_compressed];
        input.read_exact(&mut buffer).unwrap();

        let raw_buf = decompress_lzw(&buffer, block.size_raw);

        output_buf.write_all(&raw_buf).unwrap();
    }

    output_buf
}

fn decompress_lzw(input_data: &[u8], size: usize) -> Vec<u8> {
    let mut data = Cursor::new(input_data);
    let mut dictionary = HashMap::new();
    for i in 0..256 {
        dictionary.insert(i as u64, vec![i as u8]);
    }
    let mut dictionary_count = dictionary.len() as u64;
    let mut result = Vec::with_capacity(size);

    let data_size = input_data.len();

    let mut bit_io = BitReader::new(&mut data);
    let mut w = dictionary.get(&0).unwrap().clone();

    let mut element;
    loop {
        if bit_io.byte_offset() >= data_size - 1 {
            break;
        }

        let flag = bit_io.read_bit(1);
        if flag == 0 {
            element = bit_io.read_bit(15);
        } else {
            element = bit_io.read_bit(18);
        }

        let mut entry;
        if let Some(x) = dictionary.get(&element) {
            // If the element was already in the dict, get it
            entry = x.clone()
        } else if element == dictionary_count {
            entry = w.clone();
            entry.push(w[0])
        } else {
            panic!("Bad compressed element: {}", element)
        }

        result.write_all(&entry).unwrap();
        w.push(entry[0]);
        dictionary.insert(dictionary_count, w.clone());
        dictionary_count += 1;
        w.clone_from(&entry);
    }
    result
}
