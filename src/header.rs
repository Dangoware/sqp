use byteorder::{WriteBytesExt, LE};
use std::io::{Cursor, Write};

pub struct Header {
    pub magic: [u8; 8],

    /// Width of the image in pixels
    pub width: u32,
    /// Height of the image in pixels
    pub height: u32,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            magic: *b"dangoimg",
            width: 0,
            height: 0,
        }
    }
}

impl Header {
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut buf = Cursor::new(Vec::new());

        buf.write_all(&self.magic).unwrap();
        buf.write_u32::<LE>(self.width).unwrap();
        buf.write_u32::<LE>(self.height).unwrap();

        buf.into_inner().try_into().unwrap()
    }
}

pub enum ColorFormat {
    /// RGBA, 8 bits per channel
    Rgba32,

    /// RGB, 8 bits per channel
    Rgb24,
}

impl ColorFormat {
    /// Bits per color channel.
    ///
    /// Ex. Rgba32 has `8bpc`
    pub fn bpc(&self) -> u8 {
        match self {
            ColorFormat::Rgba32 => 8,
            ColorFormat::Rgb24 => 8,
        }
    }

    /// Bits per pixel.
    ///
    /// Ex. Rgba32 has `32bpp`
    pub fn bpp(&self) -> u16 {
        match self {
            ColorFormat::Rgba32 => 32,
            ColorFormat::Rgb24 => 24,
        }
    }

    /// Number of color channels.
    ///
    /// Ex. Rgba32 has `4` channels
    pub fn channels(self) -> u16 {
        match self {
            ColorFormat::Rgba32 => 4,
            ColorFormat::Rgb24 => 3,
        }
    }
}
