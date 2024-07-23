mod compression;
mod header;
mod operations;
mod binio;
mod picture;

use std::fs::File;
use header::Header;
use picture::DangoPicture;

use image::RgbaImage;

fn main() {
    let image_data = image::open("test.png").unwrap().to_rgba8();
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
    out_image.save("test.png").unwrap();
}
