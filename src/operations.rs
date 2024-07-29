use crate::ColorFormat;

pub fn sub_rows(width: u32, height: u32, color_format: ColorFormat, input: &[u8]) -> Vec<u8> {
    let mut data = Vec::with_capacity(width as usize * color_format.pbc());

    let block_height = f32::ceil(height as f32 / 3.0) as u32;
    let line_byte_count = (width * color_format.pbc() as u32) as usize;

    let mut curr_line: Vec<u8>;
    let mut prev_line: Vec<u8> = Vec::new();

    let mut i = 0;
    for y in 0..height {
        curr_line = input[i..i + line_byte_count].to_vec();

        if y % block_height != 0 {
            curr_line.iter_mut()
                .zip(prev_line.iter_mut())
                .for_each(|(curr, prev)| {
                    *curr = curr.wrapping_sub(*prev);
                    *prev = prev.wrapping_add(*curr);
                });
        } else {
            prev_line.clone_from(&curr_line);
        }

        data.extend_from_slice(&curr_line);
        i += line_byte_count;
    }

    if color_format.alpha_channel().is_some() {
        let (pixels, alpha): (Vec<&[u8]>, Vec<u8>) =
            data.chunks(color_format.pbc())
                .map(|i| (
                    &i[..color_format.pbc() - 1],
                    i[color_format.alpha_channel().unwrap()]
                ))
                .unzip();

        pixels.into_iter().flatten().copied().chain(alpha).collect()
    } else {
        data
    }
}

pub fn add_rows(width: u32, height: u32, color_format: ColorFormat, data: &[u8]) -> Vec<u8> {
    let mut output_buf = Vec::with_capacity((width * height * color_format.pbc() as u32) as usize);

    let block_height = f32::ceil(height as f32 / 3.0) as u32;

    let mut curr_line: Vec<u8>;
    let mut prev_line = Vec::new();

    let mut rgb_index = 0;
    let mut alpha_index = (width * height * (color_format.pbc() - 1) as u32) as usize;
    for y in 0..height {
        curr_line = if color_format.alpha_channel().is_some() {
            // Interleave the offset alpha into the RGB bytes
            data[rgb_index..rgb_index + width as usize * (color_format.pbc() - 1)]
                .chunks(color_format.pbc() - 1)
                .zip(data[alpha_index..alpha_index + width as usize].into_iter())
                .flat_map(|(a, b)| {
                    a.into_iter().chain(vec![b])
                })
                .copied()
                .collect()
        } else {
            data[rgb_index..rgb_index + width as usize * color_format.pbc()].to_vec()
        };

        if y % block_height != 0 {
            curr_line
                .iter_mut()
                .zip(&prev_line)
                .for_each(|(curr_p, prev_p)| {
                    *curr_p = curr_p.wrapping_add(*prev_p);
                });
        }

        // Write the decoded RGBA data to the final buffer
        output_buf.extend_from_slice(&curr_line);

        prev_line.clone_from(&curr_line);
        rgb_index += if color_format.alpha_channel().is_some() {
            width as usize * (color_format.pbc() - 1)
        } else {
            width as usize * color_format.pbc()
        };
        alpha_index += width as usize;
    }

    output_buf
}
