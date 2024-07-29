use std::{fs::File, io::{self, BufWriter, Read, Write}};

use byteorder::{ReadBytesExt, WriteBytesExt};
use integer_encoding::VarInt;
use thiserror::Error;

use crate::{
    compression::{dct::{dct_compress, dct_decompress, DctParameters},
    lossless::{compress, decompress, CompressionError, CompressionInfo}},
    header::{ColorFormat, CompressionType, Header},
    operations::{diff_line, line_diff},
};

pub struct DangoPicture {
    pub header: Header,
    pub bitmap: Vec<u8>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("incorrect identifier, got {0:02X?}")]
    InvalidIdentifier([u8; 8]),

    #[error("io operation failed: {0}")]
    IoError(#[from] io::Error),

    #[error("compression operation failed: {0}")]
    CompressionError(#[from] CompressionError),
}

impl DangoPicture {
    /// Create a DPF from raw bytes in a particular [`ColorFormat`].
    ///
    /// The quality parameter does nothing if the compression type is not
    /// lossy, so it should be set to None.
    ///
    /// ## Example
    /// ```
    /// let dpf_lossy = DangoPicture::from_raw(
    ///     input.width(),
    ///     input.height(),
    ///     ColorFormat::Rgba32,
    ///     CompressionType::LossyDct,
    ///     Some(80),
    ///     input.as_raw().clone()
    /// );
    /// ```
    pub fn from_raw(
        width: u32,
        height: u32,
        color_format: ColorFormat,
        compression_type: CompressionType,
        quality: Option<u8>,
        bitmap: Vec<u8>,
    ) -> Self {
        if quality.is_none() && compression_type == CompressionType::LossyDct {
            panic!("compression level must not be `None` when compression type is lossy")
        }

        let header = Header {
            magic: *b"dangoimg",

            width,
            height,

            compression_type,
            quality: match quality {
                Some(level) => level.clamp(1, 100),
                None => 0,
            },

            color_format,
        };

        DangoPicture {
            header,
            bitmap,
        }
    }

    /// Convenience method over [`DangoPicture::from_raw`] which creates a
    /// lossy image with a given quality.
    pub fn from_raw_lossy(
        width: u32,
        height: u32,
        color_format: ColorFormat,
        quality: u8,
        bitmap: Vec<u8>,
    ) -> Self {
        Self::from_raw(
            width,
            height,
            color_format,
            CompressionType::LossyDct,
            Some(quality),
            bitmap,
        )
    }


    pub fn from_raw_lossless(
        width: u32,
        height: u32,
        color_format: ColorFormat,
        bitmap: Vec<u8>,
    ) -> Self {
        Self::from_raw(
            width,
            height,
            color_format,
            CompressionType::Lossless,
            None,
            bitmap,
        )
    }

    /// Encode the image into anything that implements [Write]. Returns the
    /// number of bytes written.
    pub fn encode<O: Write + WriteBytesExt>(&self, mut output: O) -> Result<usize, Error> {
        let mut count = 0;

        // Write out the header
        output.write_all(&self.header.to_bytes()).unwrap();
        count += self.header.len();

        // Based on the compression type, modify the data accordingly
        let modified_data = match self.header.compression_type {
            CompressionType::None => &self.bitmap,
            CompressionType::Lossless => {
                &diff_line(self.header.width, self.header.height, &self.bitmap)
            },
            CompressionType::LossyDct => {
                &dct_compress(
                    &self.bitmap,
                    DctParameters {
                        quality: self.header.quality as u32,
                        format: self.header.color_format,
                        width: self.header.width as usize,
                        height: self.header.height as usize,
                    }
                )
                .concat()
                .into_iter()
                .flat_map(VarInt::encode_var_vec)
                .collect()
            },
        };

        // Compress the final image data using the basic LZW scheme
        let (compressed_data, compression_info) = compress(modified_data)?;

        // Write out compression info
        count += compression_info.write_into(&mut output).unwrap();

        // Write out compressed data
        output.write_all(&compressed_data).unwrap();
        count += compressed_data.len();

        Ok(count)
    }

    /// Encode and write the image out to a file.
    pub fn save<P: ?Sized + AsRef<std::path::Path>>(&self, path: &P) -> Result<(), Error> {
        let mut out_file = BufWriter::new(File::create(path.as_ref())?);

        self.encode(&mut out_file)?;

        Ok(())
    }

    /// Decode the image from anything that implements [Read]
    pub fn decode<I: Read + ReadBytesExt>(mut input: I) -> Result<DangoPicture, Error> {
        let header = Header::read_from(&mut input)?;

        let compression_info = CompressionInfo::read_from(&mut input);

        let pre_bitmap = decompress(&mut input, &compression_info);

        let bitmap = match header.compression_type {
            CompressionType::None => pre_bitmap,
            CompressionType::Lossless => {
                line_diff(header.width, header.height, &pre_bitmap)
            },
            CompressionType::LossyDct => {
                dct_decompress(
                    &decode_varint_stream(&pre_bitmap),
                    DctParameters {
                        quality: header.quality as u32,
                        format: header.color_format,
                        width: header.width as usize,
                        height: header.height as usize,
                    }
                )
            },
        };

        Ok(DangoPicture { header, bitmap })
    }
}

fn decode_varint_stream(stream: &[u8]) -> Vec<i16> {
    let mut output = Vec::new();
    let mut offset = 0;

    while let Some(num) = i16::decode_var(&stream[offset..]) {
        offset += num.1;
        output.push(num.0);
    }

    output
}
