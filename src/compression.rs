use std::{collections::HashMap, io::Write};

use byteorder::{WriteBytesExt, LE};

use crate::binio::BitIo;

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

    /// Total size of the data when compressed
    pub total_size_compressed: usize,

    /// The compression chunk information
    pub chunks: Vec<ChunkInfo>,

    /// Length of the compression chunk info
    pub length: usize,
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

pub fn compress2(data: &[u8]) -> (Vec<u8>, CompressionInfo) {
    let mut part_data;

    let mut offset = 0;
    let mut count;
    let mut last: Vec<u8> = Vec::new();

    let mut output_buf: Vec<u8> = Vec::new();
    let mut output_info = CompressionInfo {
        ..Default::default()
    };

    loop {
        (count, part_data, last) = compress_lzw2(&data[offset..], last);
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

    output_info.total_size_compressed = output_buf.len();
    (output_buf, output_info)
}

fn compress_lzw2(data: &[u8], last: Vec<u8>) -> (usize, Vec<u8>, Vec<u8>) {
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

    let mut bit_io = BitIo::new(vec![0u8; 0xF0000]);
    let write_bit = |bit_io: &mut BitIo, code: u64| {
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
        return (count, bit_io.bytes(), Vec::new());
    } else if bit_io.byte_size() < 0x87BDF {
        if !last_element.is_empty() {
            write_bit(&mut bit_io, *dictionary.get(&last_element).unwrap());
        }
        return (count, bit_io.bytes(), Vec::new());
    }

    (count, bit_io.bytes(), last_element)
}
