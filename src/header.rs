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
