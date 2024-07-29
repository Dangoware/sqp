mod compression {
    pub mod dct;
    pub mod lossless;
}
mod binio;
mod header;
mod operations;
pub mod picture;

use std::{fs::File, io::{BufReader, BufWriter}, time::Instant};
use header::{ColorFormat, CompressionType};
use image::{ImageReader, RgbaImage};
use picture::DangoPicture;

fn main() {
    let mut input = ImageReader::open("shit.png").unwrap();
    input.no_limits();
    let input = input.decode().unwrap().to_rgba8();
    input.save("original.png").unwrap();

    let dpf_lossy = DangoPicture::from_raw(
        input.width(),
        input.height(),
        ColorFormat::Rgba32,
        CompressionType::LossyDct,
        Some(80),
        input.as_raw().clone()
    );

    let dpf_lossless = DangoPicture::from_raw(
        input.width(),
        input.height(),
        ColorFormat::Rgba32,
        CompressionType::Lossless,
        None,
        input.as_raw().clone()
    );

    println!("\n--- LOSSY ---");
    println!("Encoding");
    let timer = Instant::now();
    let mut outfile = BufWriter::new(std::fs::File::create("test-lossy.dpf").unwrap());
    let size = dpf_lossy.encode(&mut outfile).unwrap();
    println!("Encoding took {}ms", timer.elapsed().as_millis());
    println!("Size is {}Mb", (((size as f32 / 1_000_000.0) * 100.0) as u32 as f32) / 100.0);

    println!("Decoding");
    let timer = Instant::now();
    let mut infile = BufReader::new(File::open("test-lossy.dpf").unwrap());
    let decoded_dpf = DangoPicture::decode(&mut infile).unwrap();
    RgbaImage::from_raw(decoded_dpf.header.width, decoded_dpf.header.height, decoded_dpf.bitmap.into()).unwrap().save("test-lossy.png").unwrap();
    println!("Decoding took {}ms", timer.elapsed().as_millis());

    println!("\n--- LOSSLESS ---");
    println!("Encoding");
    let timer = Instant::now();
    let mut outfile = BufWriter::new(std::fs::File::create("test-lossless.dpf").unwrap());
    let size = dpf_lossless.encode(&mut outfile).unwrap();
    println!("Encoding took {}ms", timer.elapsed().as_millis());
    println!("Size is {}Mb", (((size as f32 / 1_000_000.0) * 100.0) as u32 as f32) / 100.0);

    println!("Decoding");
    let timer = Instant::now();
    let mut infile = BufReader::new(File::open("test-lossless.dpf").unwrap());
    let decoded_dpf = DangoPicture::decode(&mut infile).unwrap();
    RgbaImage::from_raw(decoded_dpf.header.width, decoded_dpf.header.height, decoded_dpf.bitmap.into()).unwrap().save("test-lossless.png").unwrap();
    println!("Decoding took {}ms", timer.elapsed().as_millis());
}
