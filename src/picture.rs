use std::io::{self, Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};
use thiserror::Error;

use crate::{
    compression::{dct::{dct_compress, DctParameters}, lossless::{compress, decompress, CompressionError, CompressionInfo}},
    header::{ColorFormat, CompressionType, Header},
    operations::{diff_line, line_diff},
};

pub struct DangoPicture {
    pub header: Header,
    pub bitmap: Vec<u8>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("incorrect identifier, got {0:?}")]
    InvalidIdentifier([u8; 8]),

    #[error("io operation failed: {0}")]
    IoError(#[from] io::Error),

    #[error("compression operation failed: {0}")]
    CompressionError(#[from] CompressionError),
}

impl DangoPicture {
    pub fn from_raw(
        width: u32,
        height: u32,
        color_format: ColorFormat,
        compression_type: CompressionType,
        bitmap: Vec<u8>,
    ) -> Self {
        let header = Header {
            width,
            height,

            compression_type,
            color_format,

            ..Default::default()
        };

        DangoPicture {
            header,
            bitmap,
        }
    }

    /// Encode the image into anything that implements [Write]
    pub fn encode<O: Write + WriteBytesExt>(&self, mut output: O) -> Result<(), Error> {
        // Write out the header
        output.write_all(&self.header.to_bytes()).unwrap();

        let modified_data = match self.header.compression_type {
            CompressionType::None => &self.bitmap,
            CompressionType::Lossless => &diff_line(self.header.width, self.header.height, &self.bitmap),
            CompressionType::LossyDct => {
                &dct_compress(
                    &self.bitmap,
                    DctParameters {
                        quality: self.header.compression_level as u32,
                        format: self.header.color_format,
                        width: self.header.width as usize,
                        height: self.header.height as usize,
                    }
                ).concat().iter().flat_map(|i| i.to_le_bytes()).collect()
            },
        };

        // Compress the image data
        let (compressed_data, compression_info) = compress(&modified_data)?;

        // Write out compression info
        compression_info.write_into(&mut output).unwrap();

        // Write out compressed data
        output.write_all(&compressed_data).unwrap();

        Ok(())
    }

    /// Decode the image from anything that implements [Read]
    pub fn decode<I: Read + ReadBytesExt>(mut input: I) -> Result<DangoPicture, Error> {
        let mut magic = [0u8; 8];
        input.read_exact(&mut magic).unwrap();

        if magic != *b"dangoimg" {
            return Err(Error::InvalidIdentifier(magic));
        }

        let header = Header::read_from(&mut input)?;

        let compression_info = CompressionInfo::read_from(&mut input);

        let preprocessed_bitmap = decompress(&mut input, &compression_info);

        let bitmap = line_diff(header.width, header.height, &preprocessed_bitmap);

        Ok(DangoPicture { header, bitmap })
    }
}
