mod binio;

use std::{collections::HashMap, fs::{read, File}, io::Write};

use binio::BitIo;
use byteorder::{WriteBytesExt, LE};

fn main() {
    let image_data = read("littlespace.rgba").unwrap();
    let mut file = File::create("test.di").unwrap();
    let mut raw_diffed = File::create("test.raw").unwrap();

    let header = Header {
        magic: *b"dangoimg",
        width: 64,
        height: 64,
    };

    // Write out the header
    file.write_all(&header.to_bytes()).unwrap();

    let modified_data = diff_line(header.width, header.height, &image_data);

    raw_diffed.write_all(&modified_data).unwrap();

    // Compress the image data
    let (compressed_data, compression_info) = compress2(&modified_data);

    // Write out compression info
    compression_info.write_into(&mut file).unwrap();

    // Write out compressed data
    file.write_all(&compressed_data).unwrap();
}

struct Header {
    magic: [u8; 8],

    width: u32,
    height: u32,
}

impl Header {
    fn to_bytes(&self) -> [u8; 16] {
        let mut buf = [0u8; 16];

        buf[0..8].copy_from_slice(&self.magic);
        buf[8..12].copy_from_slice(&self.width.to_le_bytes());
        buf[12..16].copy_from_slice(&self.height.to_le_bytes());

        buf
    }
}

/*
fn diff_line(width: u32, height: u32, input: &[u8]) -> Vec<u8> {
    let mut data = Vec::with_capacity(input.len());

    let block_height = f32::ceil(height as f32 / 3.0) as usize;
    let pixel_byte_count = 32 >> 3;
    let line_byte_count = (width * pixel_byte_count as u32) as usize;

    let mut curr_line;
    let mut prev_line: Vec<u8> = Vec::with_capacity(line_byte_count);

    let mut i = 0;
    for y in 0..height {
        curr_line = input[i..i + line_byte_count].to_vec();
        if y % block_height as u32 != 0 {
            curr_line.iter_mut().zip(prev_line.iter_mut()).for_each(|(c, p)| {
                *c = c.wrapping_sub(*p);
                *p = p.wrapping_add(*c);
            });
        } else {
            prev_line.clone_from(&curr_line);
        }

        data.extend_from_slice(&curr_line);
        i += line_byte_count;
    }

    data
}
*/


fn diff_line(width: u32, height: u32, input: &[u8]) -> Vec<u8> {
    let mut data = Vec::with_capacity(width as usize * 3);
    let mut alpha_data = Vec::with_capacity(width as usize);

    let block_height = (f32::ceil(height as f32 / 3.0) as u16) as usize;
    let pixel_byte_count = 4;
    let line_byte_count = (width * pixel_byte_count as u32) as usize;

    let mut curr_line: Vec<u8>;
    let mut prev_line: Vec<u8> = Vec::with_capacity(width as usize * 3);

    let mut curr_alpha: Vec<u8>;
    let mut prev_alpha: Vec<u8> = Vec::with_capacity(width as usize);

    let mut i = 0;
    for y in 0..height {
        curr_line = input[i..i + line_byte_count]
            .windows(4)
            .step_by(4)
            .flat_map(|r| &r[0..3])
            .copied()
            .collect();
        curr_alpha = input[i..i + line_byte_count]
            .iter()
            .skip(3)
            .step_by(4)
            .copied()
            .collect();

        if y % block_height as u32 != 0 {
            for x in 0..width as usize * 3 {
                curr_line[x] = curr_line[x].wrapping_sub(prev_line[x]);
                prev_line[x] = prev_line[x].wrapping_add(curr_line[x]);
            }
            for x in 0..width as usize {
                curr_alpha[x] = curr_alpha[x].wrapping_sub(prev_alpha[x]);
                prev_alpha[x] = prev_alpha[x].wrapping_add(curr_alpha[x]);
            }
        } else {
            prev_line.clone_from(&curr_line);
            prev_alpha.clone_from(&curr_alpha);
        }

        data.extend_from_slice(&curr_line);
        alpha_data.extend_from_slice(&curr_alpha);
        i += line_byte_count;
    }

    data.extend_from_slice(&alpha_data);

    data
}


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
