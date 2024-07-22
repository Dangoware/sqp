mod binio;
mod compression;
mod header;
mod operations;

use std::{fs::{read, File}, io::Write};

use compression::compress2;
use header::Header;
use operations::diff_line;

fn main() {
    let image_data = read("littlespace.rgba").unwrap();
    let mut file = File::create("test.dpf").unwrap();

    let header = Header {
        magic: *b"dangoimg",
        width: 64,
        height: 64,
    };

    // Write out the header
    file.write_all(&header.to_bytes()).unwrap();

    let modified_data = diff_line(header.width, header.height, &image_data);

    // Compress the image data
    let (compressed_data, compression_info) = compress2(&modified_data);

    // Write out compression info
    compression_info.write_into(&mut file).unwrap();

    // Write out compressed data
    file.write_all(&compressed_data).unwrap();
}
