use std::{
    collections::HashMap,
    io::{Cursor, Read, Write},
};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use rayon::iter::{IntoParallelRefIterator, ParallelExtend, ParallelIterator};
use thiserror::Error;

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
    ) -> Result<usize, std::io::Error> {
        let mut size = 0;
        output.write_u32::<LE>(self.chunk_count as u32)?;
        size += 4;

        for chunk in &self.chunks {
            output.write_u32::<LE>(chunk.size_compressed as u32)?;
            output.write_u32::<LE>(chunk.size_raw as u32)?;
            size += 8;
        }

        Ok(size)
    }

    pub fn read_from<T: Read + ReadBytesExt>(input: &mut T) -> Self {
        let mut compression_info = CompressionInfo {
            chunk_count: input.read_u32::<LE>().unwrap() as usize,
            chunks: Vec::new(),
        };

        for _ in 0..compression_info.chunk_count {
            compression_info.chunks.push(ChunkInfo {
                size_compressed: input.read_u32::<LE>().unwrap() as usize,
                size_raw: input.read_u32::<LE>().unwrap() as usize,
            });
        }

        compression_info
    }
}

#[derive(Debug, Error)]
pub enum CompressionError {
    #[error("bad compressed element \"{1}\" at byte {2}")]
    BadElement(Vec<u8>, u64, usize),

    #[error("no chunks compressed")]
    NoChunks,
}

pub fn compress(data: &[u8]) -> Result<(Vec<u8>, CompressionInfo), CompressionError> {
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
        return Err(CompressionError::NoChunks)
    }

    Ok((output_buf, output_info))
}

fn compress_lzw(data: &[u8], last: Vec<u8>) -> (usize, Vec<u8>, Vec<u8>) {
    let mut count = 0;
    let mut dictionary: HashMap<Vec<u8>, u64> = HashMap::from_iter((0..=255).into_iter().map(|i| (vec![i], i as u64)));
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
            break;
        }
    }

    let last_element = element;
    if bit_io.byte_size() == 0 {
        if !last_element.is_empty() {
            for c in last_element {
                write_bit(&mut bit_io, *dictionary.get(&vec![c]).unwrap());
            }
        }

        bit_io.flush();
        return (count, output_buf, Vec::new());
    } else if dictionary_count < 0x3FFFE {
        if !last_element.is_empty() {
            write_bit(&mut bit_io, *dictionary.get(&last_element).unwrap());
        }

        bit_io.flush();
        return (count, output_buf, Vec::new());
    }

    bit_io.flush();
    (count, output_buf, last_element)
}

pub fn decompress<T: ReadBytesExt + Read>(
    input: &mut T,
    compression_info: &CompressionInfo
) -> Vec<u8> {
    // Read the compressd chunks from the input stream into memory
    let mut compressed_chunks = Vec::new();
    let mut total_size_raw = 0;
    for (i, block_info) in compression_info.chunks.iter().enumerate() {
        let mut buffer = vec![0u8; block_info.size_compressed];
        input.read_exact(&mut buffer).unwrap();

        compressed_chunks.push((buffer, block_info.size_raw, i));
        total_size_raw += block_info.size_raw;
    }

    // Process the compressed chunks in parallel
    let mut output_buf: Vec<u8> = Vec::with_capacity(total_size_raw);
    output_buf.par_extend(
        compressed_chunks
            .par_iter()
            .flat_map(|chunk| {
                let error = match decompress_lzw(&chunk.0, chunk.1) {
                    Ok(result) => return result,
                    Err(err) => err,
                };

                println!("{} in block {}", error, chunk.2);

                let partial = match error {
                    CompressionError::BadElement(partial, _, _) => partial,
                    _ => vec![],
                };

                let mut out = vec![0; chunk.1];

                out[..partial.len()].copy_from_slice(&partial);

                out
            })
    );

    output_buf
}

fn decompress_lzw(input_data: &[u8], size: usize) -> Result<Vec<u8>, CompressionError> {
    let mut data = Cursor::new(input_data);

    // Build the initial dictionary of 256 values
    let mut dictionary = Vec::new();
    for i in 0..256 {
        dictionary.push(vec![i as u8]);
    }
    let mut dictionary_count = dictionary.len() as u64;

    let mut result = Vec::with_capacity(size);
    let data_size = input_data.len();

    let mut bit_io = BitReader::new(&mut data);
    let mut w = dictionary.get(0).unwrap().clone();

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
        if let Some(x) = dictionary.get(element as usize) {
            // If the element was already in the dict, get it
            entry = x.clone()
        } else if element == dictionary_count {
            entry = w.clone();
            entry.push(w[0])
        } else {
            return Err(CompressionError::BadElement(result, element, bit_io.byte_offset()))
        }

        result.write_all(&entry).unwrap();
        w.push(entry[0]);
        dictionary.push(w.clone());
        dictionary_count += 1;
        w.clone_from(&entry);
    }

    Ok(result)
}
