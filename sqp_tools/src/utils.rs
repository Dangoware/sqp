use std::path::Path;

use image::ColorType;
use sqp::ColorFormat;
use text_io::read;

pub enum Assume {
    Yes,
    No,
}

pub fn color_format(s: &str) -> Result<ColorFormat, String> {
    if !s.is_ascii() {
        return Err(format!("Invalid color format {}", s))
    }

    let s_lower = s.to_lowercase();

    let color_format = match s_lower.as_str() {
        "rgba8" => ColorFormat::Rgba8,
        "rgb8" => ColorFormat::Rgb8,
        "graya8" => ColorFormat::GrayA8,
        "gray8" => ColorFormat::Gray8,
        _ => return Err(format!("Invalid color format {}", s)),
    };

    Ok(color_format)
}

pub fn color_type_to_format(img_color_format: ColorType) -> Option<ColorFormat> {
    Some(match img_color_format {
        ColorType::L8 => ColorFormat::Gray8,
        ColorType::La8 => ColorFormat::GrayA8,
        ColorType::Rgb8 => ColorFormat::Rgb8,
        ColorType::Rgba8 => ColorFormat::Rgba8,
        _ => return None,
    })
}

pub fn color_format_to_type(img_color_format: ColorFormat) -> ColorType {
    match img_color_format {
        ColorFormat::Gray8 => ColorType::L8,
        ColorFormat::GrayA8 => ColorType::La8,
        ColorFormat::Rgb8 => ColorType::Rgb8,
        ColorFormat::Rgba8 => ColorType::Rgba8,
    }
}

pub fn exists_decision<P: AsRef<Path>>(place: &str, action: &str, path: &P, assume: Option<Assume>) -> bool {
    let path = path.as_ref();

    match assume {
        Some(Assume::Yes) => return true,
        Some(Assume::No) => return false,
        None => (),
    }

    loop {
        print!("{place} file {path:?} already exists. {action}? [y/N] ");

        let opt: String = read!("{}\n");
        let opt = opt.to_lowercase();

        if !opt.is_empty() && opt == "y" {
            return true
        } else if !opt.is_empty() && opt != "y" {
            continue
        }

        if opt.is_empty() {
            return false
        }
    }
}
