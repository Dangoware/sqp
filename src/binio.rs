use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};

pub struct BitWriter<'a, O: Write + WriteBytesExt> {
    output: &'a mut O,

    current_byte: u8,

    byte_offset: usize,
    bit_offset: usize,

    byte_size: usize,
}

impl<'a, O: Write + WriteBytesExt> BitWriter<'a, O> {
    /// Create a new BitIO reader and writer over some data
    pub fn new(output: &'a mut O) -> Self {
        Self {
            output,

            current_byte: 0,

            byte_offset: 0,
            bit_offset: 0,

            byte_size: 0,
        }
    }

    /// Get the byte size of the reader
    pub fn byte_size(&self) -> usize {
        self.byte_size
    }

    /// Write some bits to the buffer
    pub fn write_bit(&mut self, data: u64, bit_len: usize) {
        if bit_len > 8 * 8 {
            panic!("Cannot write more than 64 bits at once");
        }

        if bit_len % 8 == 0 && self.bit_offset == 0 {
            self.write(data, bit_len / 8);
            return;
        }

        for i in 0..bit_len {
            let bit_value = (data >> i) & 1;

            self.current_byte &= !(1 << self.bit_offset);

            self.current_byte |= (bit_value << self.bit_offset) as u8;

            self.bit_offset += 1;
            if self.bit_offset >= 8 {
                self.byte_offset += 1;
                self.bit_offset = 0;

                self.output.write_u8(self.current_byte).unwrap();
                self.current_byte = 0;
            }
        }

        self.byte_size = self.byte_offset + (self.bit_offset + 7) / 8;
    }

    pub fn write(&mut self, data: u64, byte_len: usize) {
        if byte_len > 8 {
            panic!("Cannot write more than 8 bytes at once")
        }

        self.output.write_all(&data.to_le_bytes()[..byte_len]).unwrap();
        self.byte_offset += byte_len;

        self.byte_size = self.byte_offset + (self.bit_offset + 7) / 8;
    }
}

impl<'a, O: Write + WriteBytesExt> Drop for BitWriter<'_, O> {
    fn drop(&mut self) {
        let _ = self.output.write_u8(self.current_byte);
    }
}

pub struct BitReader<'a, I: Read + ReadBytesExt> {
    input: &'a mut I,

    current_byte: Option<u8>,

    byte_offset: usize,
    bit_offset: usize,

    byte_size: usize,
}


impl<'a, I: Read + ReadBytesExt> BitReader<'a, I> {
    /// Create a new BitIO reader and writer over some data
    pub fn new(input: &'a mut I) -> Self {
        let first = input.read_u8().unwrap();
        Self {
            input,

            current_byte: Some(first),

            byte_offset: 0,
            bit_offset: 0,

            byte_size: 0,
        }
    }

    /// Get the byte size of the reader
    pub fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    /// Get the byte size of the reader
    pub fn byte_size(&self) -> usize {
        self.byte_size
    }

    /// Read some bits from the buffer
    pub fn read_bit(&mut self, bit_len: usize) -> u64 {
        if bit_len > 8 * 8 {
            panic!("Cannot read more than 64 bits")
        }

        if bit_len % 8 == 0 && self.bit_offset == 0 {
            return self.read(bit_len / 8);
        }

        let mut result = 0;
        for i in 0..bit_len {
            let bit_value = ((self.current_byte.unwrap() as usize >> self.bit_offset) & 1) as u64;
            self.bit_offset += 1;

            if self.bit_offset == 8 {
                self.byte_offset += 1;
                self.bit_offset = 0;

                self.current_byte = Some(self.input.read_u8().unwrap());
            }

            result |= bit_value << i;
        }

        result
    }

    /// Read some bytes from the buffer
    pub fn read(&mut self, byte_len: usize) -> u64 {
        if byte_len > 8 {
            panic!("Cannot read more than 8 bytes")
        }

        if self.current_byte.is_none() {
            self.current_byte = Some(self.input.read_u8().unwrap());
        }

        let mut padded_slice = vec![0u8; byte_len];
        self.input.read_exact(&mut padded_slice).unwrap();
        self.byte_offset += byte_len;

        let extra_length = padded_slice.len() - byte_len;
        padded_slice.extend_from_slice(&vec![0u8; extra_length]);

        u64::from_le_bytes(padded_slice.try_into().unwrap())
    }
}
