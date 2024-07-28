use std::{f32::consts::{PI, SQRT_2}, sync::{Arc, Mutex}};

use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};

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

            output.push((tmp_sum + 128.0).round() as u8)
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
pub fn dct_compress(input: &[u8], parameters: DctParameters) -> Vec<Vec<i16>> {
    let new_width = parameters.width + (8 - parameters.width % 8);
    let new_height = parameters.height + (8 - parameters.width % 8);
    let quantization_matrix = quantization_matrix(parameters.quality);

    let mut dct_image = Vec::with_capacity(input.len());
    let channels: Vec<Vec<i16>> = (0..parameters.format.channels()).into_par_iter().map(|ch| {
        let channel: Vec<u8> = input.iter()
            .skip(ch as usize)
            .step_by(parameters.format.channels() as usize)
            .copied()
            .collect();
        println!("Encoding channel {ch}");

        // Create 2d array of the channel for ease of processing
        let mut img_2d: Vec<Vec<u8>> = channel.windows(parameters.width).step_by(parameters.width).map(|r| r.to_vec()).collect();
        img_2d.iter_mut().for_each(|r| r.resize(new_width, 0));
        img_2d.resize(new_height, vec![0u8; new_width]);

        let mut dct_channel = Vec::new();
        for x in 0..((new_height / 8) * (new_width / 8)) {
            let h = x / (new_width / 8);
            let w = x % (new_width / 8);

            let mut chunk = Vec::new();
            for i in 0..8 {
                let row = &img_2d[(h * 8) + i][w * 8..(w * 8) + 8];
                chunk.extend_from_slice(&row);
            }

            // Perform the DCT on the image section
            let dct: Vec<f32> = dct(&chunk, 8, 8);
            let quantized_dct = quantize(&dct, quantization_matrix);

            dct_channel.extend_from_slice(&quantized_dct);
        }

        dct_channel
    }).collect();

    channels.into_iter().for_each(|c| dct_image.push(c));

    dct_image
}

/// Take in an image encoded with DCT and quantized and perform IDCT on it,
/// returning an approximation of the original data.
pub fn dct_decompress(input: &[Vec<i16>], parameters: DctParameters) -> Vec<u8> {
    let new_width = parameters.width + (8 - parameters.width % 8);
    let new_height = parameters.height + (8 - parameters.width % 8);

    // Precalculate the quantization matrix
    let quantization_matrix = quantization_matrix(parameters.quality);

    let final_img = Arc::new(Mutex::new(vec![0u8; (new_width * new_height) * parameters.format.channels() as usize]));

    input.par_iter().enumerate().for_each(|(chan_num, channel)| {
        println!("Decoding channel {chan_num}");

        let decoded_image = Arc::new(Mutex::new(vec![0u8; parameters.width * parameters.height]));
        channel.into_par_iter().copied().chunks(64).enumerate().for_each(|(i, chunk)| {
            let dequantized_dct = dequantize(&chunk, quantization_matrix);
            let original = idct(&dequantized_dct, 8, 8);

            // Write rows of blocks
            let start_x = (i * 8) % new_width;
            let start_y = ((i * 8) / new_width) * 8;
            let start = start_x + (start_y * parameters.width);

            for row_num in 0..8 {
                if start_y + row_num >= parameters.height {
                    break;
                }

                let row_offset = row_num * parameters.width;

                let offset = if start_x + 8 >= parameters.width {
                    parameters.width % 8
                } else {
                    8
                };

                let row_data = &original[row_num * 8..(row_num * 8) + offset];
                decoded_image.lock().unwrap()[start + row_offset..start + row_offset + offset].copy_from_slice(row_data);
            }
        });

        final_img.lock().unwrap().par_iter_mut()
            .skip(chan_num)
            .step_by(parameters.format.channels() as usize)
            .zip(decoded_image.lock().unwrap().par_iter())
            .for_each(|(c, n)| *c = *n);
    });

    Arc::try_unwrap(final_img).unwrap().into_inner().unwrap()
}

/// Parameters to pass to the [`dct_compress`] function.
#[derive(Debug, Clone, Copy)]
pub struct DctParameters {
    /// A quality level from 1-100. Higher values provide better results.
    /// Default value is 80.
    pub quality: u32,

    /// The color format of the input bytes.
    ///
    /// Since DCT can only process one channel at a time, knowing the format
    /// is important.
    pub format: ColorFormat,

    /// Width of the input image
    pub width: usize,

    /// Height of the input image
    pub height: usize,
}

impl Default for DctParameters {
    fn default() -> Self {
        Self {
            quality: 80,
            format: ColorFormat::Rgba32,
            width: 0,
            height: 0,
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
    fn run_dct() {
        let result = dct(
            &[
                6, 4, 4, 6, 10, 16, 20, 24,
                5, 5, 6, 8, 10, 23, 24, 22,
                6, 5, 6, 10, 16, 23, 28, 22,
                6, 7, 9, 12, 20, 35, 32, 25,
                7, 9, 15, 22, 27, 44, 41, 31,
                10, 14, 22, 26, 32, 42, 45, 37,
                20, 26, 31, 35, 41, 48, 48, 40,
                29, 37, 38, 39, 45, 40, 41, 40
            ],
            8,
            8
        );

        assert_eq!(
            result,
            [-839.37494, -66.86765, -5.8187184, 12.086508, -12.37503, 3.744713, 0.65127736, -1.4721011, -78.0333, -0.8744621, 14.815389, 1.9330482, 2.5059338, 1.8356638, 2.3859768, -2.1098928, 12.556393, 17.50461, 3.9685955, -8.910822, 6.42554, -4.6883383, -2.441934, 2.3615432, -1.4457717, -11.20282, -0.6175499, -0.24921608, -1.3332539, 2.59305, 2.0981073, -1.1885407, 0.6249629, 4.1257324, 0.21936417, 0.5029774, 1.625, -2.7071304, 0.8562317, -0.67780924, -0.47140676, -1.1953268, 0.7938299, 1.343049, 0.4363842, -0.75078535, -0.3206334, 1.0701582, -3.9833553, 2.071165, 1.5580511, -2.9571223, 3.426909, -0.45216227, -2.2185893, 3.0024266, 2.9214313, -0.85989547, -1.5205104, 0.891371, 0.9026685, 1.3169396, -1.0526512, -0.12552339]
        );
    }

    #[test]
    fn run_idct() {
        let result = idct(
            &[-839.37494, -66.86765, -5.8187184, 12.086508, -12.37503, 3.744713, 0.65127736, -1.4721011, -78.0333, -0.8744621, 14.815389, 1.9330482, 2.5059338, 1.8356638, 2.3859768, -2.1098928, 12.556393, 17.50461, 3.9685955, -8.910822, 6.42554, -4.6883383, -2.441934, 2.3615432, -1.4457717, -11.20282, -0.6175499, -0.24921608, -1.3332539, 2.59305, 2.0981073, -1.1885407, 0.6249629, 4.1257324, 0.21936417, 0.5029774, 1.625, -2.7071304, 0.8562317, -0.67780924, -0.47140676, -1.1953268, 0.7938299, 1.343049, 0.4363842, -0.75078535, -0.3206334, 1.0701582, -3.9833553, 2.071165, 1.5580511, -2.9571223, 3.426909, -0.45216227, -2.2185893, 3.0024266, 2.9214313, -0.85989547, -1.5205104, 0.891371, 0.9026685, 1.3169396, -1.0526512, -0.12552339],
            8,
            8
        );

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
    fn create_quantization_matrix_q80() {
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
    fn create_quantization_matrix_q100() {
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
