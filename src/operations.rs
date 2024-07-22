pub fn diff_line(width: u32, height: u32, input: &[u8]) -> Vec<u8> {
    let mut data = Vec::with_capacity(width as usize * 3);
    let mut alpha_data = Vec::with_capacity(width as usize);

    let block_height = (f32::ceil(height as f32 / 3.0) as u16) as usize;
    let pixel_byte_count = 4;
    let line_byte_count = (width * pixel_byte_count as u32) as usize;

    let mut curr_line: Vec<u8>;
    let mut prev_line: Vec<u8> = Vec::with_capacity(width as usize * 3);

    let mut curr_alpha: Vec<u8>;
    let mut prev_alpha: Vec<u8> = Vec::with_capacity(width as usize);

    let mut i = 0;
    for y in 0..height {
        curr_line = input[i..i + line_byte_count]
            .windows(4)
            .step_by(4)
            .flat_map(|r| &r[0..3])
            .copied()
            .collect();
        curr_alpha = input[i..i + line_byte_count]
            .iter()
            .skip(3)
            .step_by(4)
            .copied()
            .collect();

        if y % block_height as u32 != 0 {
            for x in 0..width as usize * 3 {
                curr_line[x] = curr_line[x].wrapping_sub(prev_line[x]);
                prev_line[x] = prev_line[x].wrapping_add(curr_line[x]);
            }
            for x in 0..width as usize {
                curr_alpha[x] = curr_alpha[x].wrapping_sub(prev_alpha[x]);
                prev_alpha[x] = prev_alpha[x].wrapping_add(curr_alpha[x]);
            }
        } else {
            prev_line.clone_from(&curr_line);
            prev_alpha.clone_from(&curr_alpha);
        }

        data.extend_from_slice(&curr_line);
        alpha_data.extend_from_slice(&curr_alpha);
        i += line_byte_count;
    }

    data.extend_from_slice(&alpha_data);

    data
}
