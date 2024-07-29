use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{Cursor, Read, Write};

use crate::picture::Error;

/// A DPF file header. This must be included at the beginning
/// of a valid DPF file.
pub struct Header {
    /// Identifier. Must be set to "dangoimg".
    pub magic: [u8; 8],

    /// Width of the image in pixels.
    pub width: u32,

    /// Height of the image in pixels.
    pub height: u32,

    /// Type of compression used on the data.
    pub compression_type: CompressionType,

    /// Level of compression. Only applies in Lossy mode, otherwise this value
    /// should be set to 0, and ignored.
    pub quality: u8,

    /// Format of color data in the image.
    pub color_format: ColorFormat,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            magic: *b"dangoimg",
            width: 0,
            height: 0,
            compression_type: CompressionType::Lossless,
            quality: 0,
            color_format: ColorFormat::Rgba8,
        }
    }
}

impl Header {
    pub fn to_bytes(&self) -> [u8; 19] {
        let mut buf = Cursor::new(Vec::new());

        buf.write_all(&self.magic).unwrap();
        buf.write_u32::<LE>(self.width).unwrap();
        buf.write_u32::<LE>(self.height).unwrap();

        // Write compression info
        buf.write_u8(self.compression_type.into()).unwrap();
        buf.write_u8(self.quality).unwrap();

        // Write color format
        buf.write_u8(self.color_format as u8).unwrap();

        buf.into_inner().try_into().unwrap()
    }

    /// Length of the header in bytes.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        19
    }

    /// Create a header from something implementing [`Read`].
    pub fn read_from<T: Read + ReadBytesExt>(input: &mut T) -> Result<Self, Error> {
        let mut magic = [0u8; 8];
        input.read_exact(&mut magic).unwrap();

        if magic != *b"dangoimg" {
            return Err(Error::InvalidIdentifier(magic));
        }

        Ok(Header {
            magic,
            width: input.read_u32::<LE>()?,
            height: input.read_u32::<LE>()?,

            compression_type: input.read_u8()?.try_into().unwrap(),
            quality: input.read_u8()?,
            color_format: input.read_u8()?.try_into().unwrap(),
        })
    }
}

/// The format of bytes in the image.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    /// RGBA, 8 bits per channel
    Rgba8 = 0,

    /// RGB, 8 bits per channel
    Rgb8 = 1,
}

impl ColorFormat {
    /// Bits per color channel.
    ///
    /// Ex. Rgba32 has `8bpc`
    pub fn bpc(&self) -> u8 {
        match self {
            ColorFormat::Rgba8 => 8,
            ColorFormat::Rgb8 => 8,
        }
    }

    /// Bits per pixel.
    ///
    /// Ex. Rgba32 has `32bpp`
    pub fn bpp(&self) -> u16 {
        match self {
            ColorFormat::Rgba8 => 32,
            ColorFormat::Rgb8 => 24,
        }
    }

    /// Number of color channels.
    ///
    /// Ex. Rgba32 has `4` channels
    pub fn channels(self) -> u16 {
        match self {
            ColorFormat::Rgba8 => 4,
            ColorFormat::Rgb8 => 3,
        }
    }
}

impl TryFrom<u8> for ColorFormat {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Rgba8,
            1 => Self::Rgb8,
            v => return Err(format!("invalid color format {v}")),
        })
    }
}

/// The type of compression used in the image
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// No compression at all, raw bitmap
    None = 0,

    /// Lossless compression
    Lossless = 1,

    /// Lossy Discrete Cosine Transform compression
    LossyDct = 2,
}

impl TryFrom<u8> for CompressionType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::None,
            1 => Self::Lossless,
            2 => Self::LossyDct,
            v => return Err(format!("invalid compression type {v}"))
        })
    }
}

impl From<CompressionType> for u8 {
    fn from(val: CompressionType) -> Self {
        match val {
            CompressionType::None => 0,
            CompressionType::Lossless => 1,
            CompressionType::LossyDct => 2,
        }
    }
}
