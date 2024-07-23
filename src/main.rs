mod compression;
mod header;
mod operations;
mod binio;

use std::{fs::File, io::{Read, Write}};

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use compression::{compress, decompress, ChunkInfo, CompressionInfo};
use header::Header;
use image::RgbaImage;
use operations::{diff_line, line_diff};

fn main() {
    let image_data = image::open("dripping.png").unwrap().to_rgba8();
    let encoded_dpf = DangoPicture {
        header: Header {
            width: image_data.width(),
            height: image_data.height(),

            ..Default::default()
        },
        bitmap: image_data.into_vec(),
    };

    let mut outfile = File::create("test.dpf").unwrap();
    encoded_dpf.encode(&mut outfile);


    let mut infile = File::open("test.dpf").unwrap();
    let decoded_dpf = DangoPicture::decode(&mut infile);
    let out_image = RgbaImage::from_raw(
        decoded_dpf.header.width,
        decoded_dpf.header.height,
        decoded_dpf.bitmap
    ).unwrap();
    out_image.save("test2.png").unwrap();
}

struct DangoPicture {
    header: Header,
    bitmap: Vec<u8>,
}

impl DangoPicture {
    fn encode<O: Write + WriteBytesExt>(&self, mut output: O) {
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

    fn decode<I: Read + ReadBytesExt>(mut input: I) -> DangoPicture {
        let mut magic = [0u8; 8];
        input.read_exact(&mut magic).unwrap();

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

        DangoPicture {
            header,
            bitmap
        }
    }
}
