use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use thiserror::Error;

use crate::{compression::{compress, decompress, ChunkInfo, CompressionInfo}, header::Header, operations::{diff_line, line_diff}};

pub struct DangoPicture {
    pub header: Header,
    pub bitmap: Vec<u8>,
}

impl DangoPicture {
    /// Encode the image into anything that implements [Write]
    pub fn encode<O: Write + WriteBytesExt>(&self, mut output: O) {
        let header = Header {
            width: self.header.width,
            height: self.header.height,

            ..Default::default()
        };

        // Write out the header
        output.write_all(&header.to_bytes()).unwrap();

        let modified_data = diff_line(header.width, header.height, &self.bitmap);

        // Compress the image data
        let (compressed_data, compression_info) = compress(&modified_data);

        // Write out compression info
        compression_info.write_into(&mut output).unwrap();

        // Write out compressed data
        output.write_all(&compressed_data).unwrap();
    }

    /// Decode the image from anything that implements [Read]
    pub fn decode<I: Read + ReadBytesExt>(mut input: I) -> Result<DangoPicture, Error> {
        let mut magic = [0u8; 8];
        input.read_exact(&mut magic).unwrap();

        if magic != *b"dangoimg" {
            return Err(Error::InvalidIdentifier(magic))
        }

        let header = Header {
            magic,
            width: input.read_u32::<LE>().unwrap(),
            height: input.read_u32::<LE>().unwrap(),
        };

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

        let preprocessed_bitmap = decompress(&mut input, &compression_info);

        let bitmap = line_diff(header.width, header.height, &preprocessed_bitmap);

        Ok(DangoPicture {
            header,
            bitmap
        })
    }

    pub fn from_raw(width: u32, height: u32, bitmap: &[u8]) -> Self {
        let header = Header {
            width,
            height,

            ..Default::default()
        };

        DangoPicture {
            header,
            bitmap: bitmap.into(),
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("incorrect identifier, got {}", 0)]
    InvalidIdentifier([u8; 8]),
}
