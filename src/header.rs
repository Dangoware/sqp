use std::io::{Cursor, Write};
use byteorder::{WriteBytesExt, LE};

pub struct Header {
    pub magic: [u8; 8],

    /// Width of the image in pixels
    pub width: u32,
    /// Height of the image in pixels
    pub height: u32,

    /// Bit depth in bits per pixel
    pub depth: u16,

    pub encoding: ImageEncoding,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            magic: *b"dangoimg",
            width: 0,
            height: 0,
            depth: 32,
            encoding: ImageEncoding::LosslessCompressed,
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

#[repr(u16)]
pub enum ImageEncoding {
    /// Uncompressed raw bitmap
    Bitmap = 0,

    /// Losslessly compressed
    LosslessCompressed = 1,

    /// Lossily compressed
    LossyCompressed = 2,
}
