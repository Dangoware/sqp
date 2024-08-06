//! Structs and enums which are included in the header of SQP files.

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, Read, Write};

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
    /// Write the header into a byte stream implementing [`Write`].
    ///
    /// Returns the number of bytes written.
    pub fn write_into<W: Write + WriteBytesExt>(&self, output: &mut W) -> Result<usize, io::Error> {
        let mut count = 0;
        output.write_all(&self.magic)?;
        output.write_u32::<LE>(self.width)?;
        output.write_u32::<LE>(self.height)?;
        count += 16;

        // Write compression info
        output.write_u8(self.compression_type.into())?;
        output.write_u8(self.quality)?;
        count += 2;

        // Write color format
        output.write_u8(self.color_format as u8)?;
        count += 1;

        Ok(count)
    }

    /// Length of the header in bytes.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        19
    }

    /// Create a header from a byte stream implementing [`Read`].
    pub fn read_from<R: Read + ReadBytesExt>(input: &mut R) -> Result<Self, Error> {
        let mut magic = [0u8; 8];
        input.read_exact(&mut magic).unwrap();

        if magic != *b"dangoimg" {
            let bad_id = String::from_utf8_lossy(&magic).into_owned();
            return Err(Error::InvalidIdentifier(bad_id));
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

    /// Grayscale with alpha, 8 bits per channel
    GrayA8 = 2,

    /// Grayscale, 8 bits per channel
    Gray8 = 3,
}

impl ColorFormat {
    /// Bits per color channel.
    ///
    /// Ex. `Rgba8` has `8bpc`
    pub fn bpc(&self) -> u8 {
        match self {
            Self::Rgba8 => 8,
            Self::Rgb8 => 8,
            Self::GrayA8 => 8,
            Self::Gray8 => 8,
        }
    }

    /// Bits per pixel.
    ///
    /// Ex. `Rgba8` has `32bpp`
    pub fn bpp(&self) -> u16 {
        match self {
            Self::Rgba8 => 32,
            Self::Rgb8 => 24,
            Self::GrayA8 => 16,
            Self::Gray8 => 8,
        }
    }

    /// Number of color channels.
    ///
    /// Ex. `Rgba8` has `4` channels
    pub fn channels(&self) -> u16 {
        match self {
            Self::Rgba8 => 4,
            Self::Rgb8 => 3,
            Self::GrayA8 => 2,
            Self::Gray8 => 1,
        }
    }

    /// The channel in which alpha is contained, or [`None`] if there is none.
    ///
    /// Ex. `Rgba8`'s 3rd channel is alpha
    pub fn alpha_channel(&self) -> Option<usize> {
        match self {
            Self::Rgba8 => Some(3),
            Self::Rgb8 => None,
            Self::GrayA8 => Some(1),
            Self::Gray8 => None,
        }
    }

    /// Pixel Byte Count, The number of bytes per pixel.
    ///
    /// Convenience method over [`Self::bpp`]
    pub fn pbc(&self) -> usize {
        (self.bpp() / 8).into()
    }
}

impl TryFrom<u8> for ColorFormat {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Rgba8,
            1 => Self::Rgb8,
            2 => Self::GrayA8,
            3 => Self::Gray8,
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
