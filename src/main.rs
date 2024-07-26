mod compression {
    pub mod dct;
    pub mod lossless;
}
mod binio;
mod header;
mod operations;
pub mod picture;

use header::ColorFormat;
use picture::DangoPicture;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    time::Instant,
};
use compression::{dct::{dct, dct_compress, dequantize, idct, quantization_matrix, quantize, DctParameters}, lossless};

use image::{GenericImage, GrayImage, Luma};

fn main() {
    let input = image::open("test_input.png").unwrap().to_luma8();
    input.save("original.png").unwrap();

    let dct_result = dct_compress(
        input.as_raw(),
        input.width(),
        input.height(),
        DctParameters {
            quality: 100,
            format: ColorFormat::Rgba32,
        }
    );

    let mut decoded_image = GrayImage::new(dct_result.width, dct_result.height);
    for (i, chunk) in dct_result.channels[0].windows(64).step_by(64).enumerate() {
        let dequantized_dct = dequantize(chunk, quantization_matrix(100));
        let original = idct(&dequantized_dct, 8, 8);

        // Write rows of blocks
        let start_x = (i * 8) % dct_result.width as usize;
        let start_y = ((i * 8) / dct_result.width as usize) * 8;

        let mut sub = decoded_image.sub_image(start_x as u32, start_y as u32, 8, 8);
        for y in 0..8 {
            for x in 0..8 {
                let value = original[(y as usize * 8) + x as usize];
                sub.put_pixel(x, y, Luma([value]))
            }
        }

        decoded_image.save(format!("test.png")).unwrap();
    }
    decoded_image.save(format!("test.png")).unwrap();

    /*
    // Reverse the DCT
    let idct: Vec<u8> = idct(&dct, 8, 8).iter().map(|c| *c as u8).collect();

    let img = GrayImage::from_raw(input.width(), input.height(), idct).unwrap();
    img.save("test.png").unwrap();
    */

    /*
    let image_data = image::open("bw.jpg").unwrap().to_rgba8();
    let encoded_dpf = DangoPicture::from_raw(image_data.width(), image_data.height(), &image_data);

    println!("ENCODING ---");
    let timer = Instant::now();
    let mut outfile = BufWriter::new(File::create("test.dpf").unwrap());
    encoded_dpf.encode(&mut outfile);
    println!("Encoding took {}ms", timer.elapsed().as_millis());

    println!("DECODING ---");
    let timer = Instant::now();
    let mut infile = BufReader::new(File::open("test.dpf").unwrap());
    let decoded_dpf = DangoPicture::decode(&mut infile).unwrap();
    println!("Decoding took {}ms", timer.elapsed().as_millis());

    let out_image = RgbaImage::from_raw(
        decoded_dpf.header.width,
        decoded_dpf.header.height,
        decoded_dpf.bitmap.into(),
    )
    .unwrap();
    out_image.save("test.png").unwrap();
    */
}
