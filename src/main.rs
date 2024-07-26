mod compression {
    pub mod dct;
    pub mod lossless;
}
mod binio;
mod header;
mod operations;
pub mod picture;

use header::ColorFormat;
use compression::dct::{dct_compress, dequantize, idct, quantization_matrix, DctParameters};

use image::{GenericImage, GrayImage, Luma, RgbaImage};

fn main() {
    let input = image::open("shit.png").unwrap().to_rgba8();
    input.save("original.png").unwrap();

    let dct_result = dct_compress(
        input.as_raw(),
        input.width(),
        input.height(),
        DctParameters {
            quality: 30,
            format: ColorFormat::Rgba32,
        }
    );

    let mut final_img = vec![0u8; (dct_result.width as usize * dct_result.height as usize) * 4];
    for (chan_num, channel) in dct_result.channels.iter().enumerate() {
        let mut decoded_image = GrayImage::new(dct_result.width, dct_result.height);
        for (i, chunk) in channel.windows(64).step_by(64).enumerate() {
            let dequantized_dct = dequantize(chunk, quantization_matrix(30));
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
        }
        final_img.iter_mut().skip(chan_num).step_by(4).zip(decoded_image.iter()).for_each(|(c, n)| *c = *n);
        decoded_image.save(format!("dct-{chan_num}.png")).unwrap();
    }
    RgbaImage::from_raw(
        dct_result.width,
        dct_result.height,
        final_img
    ).unwrap().save("dct.png").unwrap();

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
