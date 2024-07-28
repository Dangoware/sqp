mod compression {
    pub mod dct;
    pub mod lossless;
}
mod binio;
mod header;
mod operations;
pub mod picture;

use std::{fs::File, io::Write, time::Instant};

use header::ColorFormat;
use compression::{dct::{dct_compress, dct_decompress, DctParameters}, lossless};

use image::RgbaImage;
use picture::DangoPicture;

fn main() {
    let input = image::open("kirara_motorbike.jpg").unwrap().to_rgba8();
    input.save("original.png").unwrap();

    let dct_output = File::create("test.dpf").unwrap();
    DangoPicture::from_raw(input.width(), input.height(), &input.as_raw().clone()).encode(&dct_output);

    let timer = Instant::now();
    let dct_result = dct_compress(
        input.as_raw(),
        DctParameters {
            quality: 30,
            format: ColorFormat::Rgba32,
            width: input.width() as usize,
            height: input.height() as usize,
        }
    );
    println!("Encoding took {}ms", timer.elapsed().as_millis());

    let mut dct_output = File::create("test-dct.dpf").unwrap();
    let compressed_dct = lossless::compress(&dct_result.channels.concat().iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>());
    dct_output.write_all(&compressed_dct.0).unwrap();

    let timer = Instant::now();
    let decoded_dct = dct_decompress(
        &dct_result.channels,
        DctParameters {
            quality: 30,
            format: ColorFormat::Rgba32,
            width: dct_result.width as usize,
            height: dct_result.height as usize
        }
    );
    println!("Decoding took {}ms", timer.elapsed().as_millis());

    RgbaImage::from_raw(
        dct_result.width,
        dct_result.height,
        decoded_dct
    ).unwrap().save("dct-final.png").unwrap();

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
