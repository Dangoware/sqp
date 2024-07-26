use std::f32::consts::{PI, SQRT_2};

use crate::header::ColorFormat;

/// Perform a Discrete Cosine Transform on the input matrix.
pub fn dct(input: &[u8], width: usize, height: usize) -> Vec<f32> {
    if input.len() != width * height {
        panic!("Input matrix size must be width×height")
    }

    let sqrt_width_zero = 1.0 / (width as f32).sqrt();
    let sqrt_width = SQRT_2 / (width as f32).sqrt();

    let sqrt_height_zero = 1.0 / (height as f32).sqrt();
    let sqrt_height = SQRT_2 / (height as f32).sqrt();

    let mut output = Vec::new();
    for u in 0..width {
        for v in 0..height {

            // according to the formula of DCT
            let cu = if u == 0 {
                sqrt_width_zero
            } else {
                sqrt_width
            };

            let cv = if v == 0 {
                sqrt_height_zero
            } else {
                sqrt_height
            };

            // calculate DCT
            let mut tmp_sum = 0.0;
            for x in 0..width {
                for y in 0..height {
                    let dct = (input[x * width + y] as f32 - 128.0) *
                        f32::cos((2.0 * x as f32 + 1.0) * u as f32 * PI / (2.0 * width as f32)) *
                        f32::cos((2.0 * y as f32 + 1.0) * v as f32 * PI / (2.0 * height as f32));

                    tmp_sum += dct;
                }
            }

            output.push(cu * cv * tmp_sum)
        }
    }

    output
}

/// Perform an inverse Discrete Cosine Transform on the input matrix.
pub fn idct(input: &[f32], width: usize, height: usize) -> Vec<u8> {
    if input.len() != width * height {
        panic!("Input matrix size must be width×height")
    }

    let sqrt_width_zero = 1.0 / (width as f32).sqrt();
    let sqrt_width = SQRT_2 / (width as f32).sqrt();

    let sqrt_height_zero = 1.0 / (height as f32).sqrt();
    let sqrt_height = SQRT_2 / (height as f32).sqrt();

    let mut output = Vec::new();
    for x in 0..width {
        for y in 0..height {

            let mut tmp_sum = 0.0;
            for u in 0..width {
                for v in 0..height {
                    let cu = if u == 0 {
                        sqrt_width_zero
                    } else {
                        sqrt_width
                    };

                    let cv = if v == 0 {
                        sqrt_height_zero
                    } else {
                        sqrt_height
                    };

                    let idct = input[u * width + v] as f32 *
                        f32::cos((2.0 * x as f32 + 1.0) * u as f32 * PI / (2.0 * width as f32)) *
                        f32::cos((2.0 * y as f32 + 1.0) * v as f32 * PI / (2.0 * height as f32));

                    tmp_sum += cu * cv * idct
                }
            }

            output.push((tmp_sum + 128.0) as u8)
        }
    }

    output
}

/// JPEG 8x8 Base Quantization Matrix for a quality level of 50.
///
/// Instead of using this, utilize the [`quantization_matrix`] function to
/// get a quantization matrix corresponding to the image quality value.
const BASE_QUANTIZATION_MATRIX: [u16; 64] = [
    16, 11, 10, 16,  24,  40,  51,  61,
    12, 12, 14, 19,  26,  58,  60,  55,
    14, 13, 16, 24,  40,  57,  69,  56,
    14, 17, 22, 29,  51,  87,  80,  62,
    18, 22, 37, 56,  68, 109, 103,  77,
    24, 35, 55, 64,  81, 104, 113,  92,
    49, 64, 78, 87, 103, 121, 120, 101,
    72, 92, 95, 98, 112, 100, 103,  99,
];

/// Generate the 8x8 quantization matrix for the given quality level.
pub fn quantization_matrix(quality: u32) -> [u16; 64] {
    let factor = if quality < 50 {
        5000.0 / quality as f32
    } else {
        200.0 - 2.0 * quality as f32
    };

    let new_matrix = BASE_QUANTIZATION_MATRIX.map(|i|
        f32::floor((factor * i as f32 + 50.0) / 100.0) as u16
    );
    new_matrix.map(|i| if i == 0 { 1 } else { i })
}

/// Quantize an input matrix, returning the result.
pub fn quantize(input: &[f32], quant_matrix: [u16; 64]) -> Vec<i16> {
    input.iter().zip(quant_matrix).map(|(v, q)| (v / q as f32).round() as i16).collect()
}

/// Dequantize an input matrix, returning an approximation of the original.
pub fn dequantize(input: &[i16], quant_matrix: [u16; 64]) -> Vec<f32> {
    input.iter().zip(quant_matrix).map(|(v, q)| (*v as i16 * q as i16) as f32).collect()
}

/// Take in an image encoded in some [`ColorFormat`] and perform DCT on it,
/// returning the modified data. This function also pads the image dimensions
/// to a multiple of 8, which must be reversed when decoding.
pub fn dct_compress(input: &[u8], width: u32, height: u32, parameters: DctParameters) -> DctImage {
    let new_width = width as usize + (8 - width % 8) as usize;
    let new_height = height as usize + (8 - height % 8) as usize;
    let mut img_2d: Vec<Vec<u8>> = input.windows(width as usize).step_by(width as usize).map(|r| r.to_vec()).collect();
    img_2d.iter_mut().for_each(|r| r.resize(new_width, 0));
    img_2d.resize(new_height, vec![0u8; new_width]);

    let mut dct_image = Vec::new();
    for _ in 0..1 {
        let mut dct_channel = Vec::new();
        for h in 0..new_height / 8 {
            for w in 0..new_width / 8 {
                let mut chunk = Vec::new();
                for i in 0..8 {
                    let row = &img_2d[(h * 8) + i][w * 8..(w * 8) + 8];
                    chunk.extend_from_slice(&row);
                }

                // Perform the DCT on the image section
                let dct: Vec<f32> = dct(&chunk, 8, 8);
                let quantzied_dct = quantize(&dct, quantization_matrix(parameters.quality));

                dct_channel.extend_from_slice(&quantzied_dct);
            }
        }
        dct_image.push(dct_channel);
    }

    DctImage {
        channels: dct_image,
        width: new_width as u32,
        height: new_height as u32
    }
}

/// Parameters to pass to the [`dct_compress`] function.
pub struct DctParameters {
    /// A quality level from 1-100. Higher values provide better results.
    /// Default value is 80.
    pub quality: u32,

    /// The color format of the input bytes.
    ///
    /// Since DCT can only process one channel at a time, knowing the format
    /// is important.
    pub format: ColorFormat,
}

impl Default for DctParameters {
    fn default() -> Self {
        Self {
            quality: 80,
            format: ColorFormat::Rgba32
        }
    }
}

/// The results of DCT compression
pub struct DctImage {
    /// The DCT encoded version of each channel.
    pub channels: Vec<Vec<i16>>,

    /// New width after padding.
    pub width: u32,

    /// New height after padding.
    pub height: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantization_matrix_q80() {
        let result = quantization_matrix(80);

        assert_eq!(
            result,
            [
                6, 4, 4, 6, 10, 16, 20, 24,
                5, 5, 6, 8, 10, 23, 24, 22,
                6, 5, 6, 10, 16, 23, 28, 22,
                6, 7, 9, 12, 20, 35, 32, 25,
                7, 9, 15, 22, 27, 44, 41, 31,
                10, 14, 22, 26, 32, 42, 45, 37,
                20, 26, 31, 35, 41, 48, 48, 40,
                29, 37, 38, 39, 45, 40, 41, 40
            ]
        );
    }

    #[test]
    fn quantization_matrix_q100() {
        let result = quantization_matrix(100);

        assert_eq!(
            result,
            [
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1
            ]
        );
    }
}
