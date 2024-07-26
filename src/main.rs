mod compression {
    pub mod dct;
    pub mod lossless;
}
mod binio;
mod header;
mod operations;
pub mod picture;

use picture::DangoPicture;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    time::Instant,
};
use compression::{dct::{dct, dct_compress, dequantize, idct, quantization_matrix, quantize}, lossless};

use image::{ColorType, DynamicImage, GenericImage, GrayImage, Luma, Rgba};

fn main() {
    let input = image::open("transparent.png").unwrap().to_luma8();
    input.save("original.png").unwrap();

    let (dct_image, new_width, new_height) = dct_compress(input.as_raw(), input.width(), input.height(), 100);
    let compressed_dct = lossless::compress(&dct_image.iter().flatten().flat_map(|b| b.to_le_bytes()).collect::<Vec<u8>>());
    let mut dct_save = File::create("dct_raw.dct").unwrap();
    dct_save.write_all(&compressed_dct.0).unwrap();

    let mut decoded_image = GrayImage::new(new_width as u32, new_height as u32);
    for (i, chunk) in dct_image.iter().enumerate() {
        let dequantized_dct = dequantize(chunk, quantization_matrix(100));
        let original = idct(&dequantized_dct, 8, 8);

        // Write rows of blocks
        let start_x = (i * 8) % (new_width as usize);
        let start_y = ((i * 8) / new_width as usize) * 8;

        let mut sub = decoded_image.sub_image(start_x as u32, start_y as u32, 8, 8);
        for y in 0..8 {
            for x in 0..8 {
                let value = original[(y as usize * 8) + x as usize];
                sub.put_pixel(x, y, Luma([value]))
            }
        }
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
