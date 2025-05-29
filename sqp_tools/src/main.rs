mod utils;

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use image::ImageReader;
use anyhow::{bail, Result};
use sqp::{ColorFormat, CompressionType};
use utils::{color_format, color_format_to_type, color_type_to_format, exists_decision, Assume};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Subcommands,

    /// Overwrite output files
    #[arg(short = 'n', long = "overwrite", conflicts_with = "assumeno")]
    assumeyes: bool,

    /// Do not overwrite output files
    #[arg(short = 'y', long = "preserve", conflicts_with = "assumeyes")]
    assumeno: bool,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
    /// Encode an image to SQP format
    Encode(EncodeArgs),

    /// Decode an SQP image into another format
    Decode(DecodeArgs),
}

#[derive(Debug, Args)]
struct EncodeArgs {
    /// Input image file of any type supported by `image`
    input: PathBuf,
    /// Output path to SQP location
    output: PathBuf,

    /// Quality setting, a higher value = higher quality.
    #[arg(default_value_t = 100, short, long, conflicts_with = "uncompressed", value_parser = clap::value_parser!(u8).range(1..=100))]
    quality: u8,

    /// Create an uncompressed image.
    ///
    /// Incompatible with quality setting.
    #[arg(short, long, conflicts_with = "quality")]
    uncompressed: bool,

    /// The color format to use for the output image
    ///
    /// Valid values:
    ///  - RGBA8
    ///  - RGB8
    ///  - GrayA8
    ///  - Gray8
    #[arg(short, long, value_parser = color_format, verbatim_doc_comment)]
    color_format: Option<ColorFormat>,
}

#[derive(Debug, Args)]
struct DecodeArgs {
    /// Input SQP image file
    input: PathBuf,

    /// Output image file
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let assume = if args.assumeyes {
        Some(Assume::Yes)
    } else if args.assumeno {
        Some(Assume::No)
    } else {
        None
    };

    match args.command {
        Subcommands::Encode(a) => encode(a, assume),
        Subcommands::Decode(a) => decode(a, assume),
    }
}

fn encode(args: EncodeArgs, assume: Option<Assume>) -> Result<()> {
    if !args.input.try_exists()? {
        bail!("Input file {:?} does not exist", args.input);
    }

    if args.output.try_exists()?
        && !exists_decision("Output", "Overwrite", &args.output, assume)
    {
        return Ok(())
    }

    let image = ImageReader::open(args.input)?
        .decode()?;

    let width = image.width();
    let height = image.height();

    let color_format = args.color_format.or_else(
        || color_type_to_format(image.color())
    ).unwrap_or(ColorFormat::Rgba8);

    let bitmap = match color_format {
        ColorFormat::Rgba8 => image.into_rgba8().into_vec(),
        ColorFormat::Rgb8 => image.into_rgb8().into_vec(),
        ColorFormat::GrayA8 => image.into_luma_alpha8().into_vec(),
        ColorFormat::Gray8 => image.into_luma8().into_vec(),
    };

    let (compression_type, quality) = if args.uncompressed {
        (CompressionType::None, None)
    } else if args.quality == 100 {
        (CompressionType::Lossless, None)
    } else {
        (CompressionType::LossyDct, Some(args.quality))
    };

    let sqp_img = sqp::SquishyPicture::from_raw(
        width,
        height,
        color_format,
        compression_type,
        quality,
        bitmap,
    );

    sqp_img.save(&args.output)?;

    Ok(())
}

fn decode(args: DecodeArgs, assume: Option<Assume>) -> Result<()> {
    if !args.input.try_exists()? {
        bail!("Input file {:?} does not exist", args.input);
    }

    if args.output.try_exists()?
        && !exists_decision("Output", "Overwrite", &args.output, assume)
    {
        return Ok(())
    }

    let sqp_img = sqp::open(args.input)?;

    let width = sqp_img.width();
    let height = sqp_img.height();
    let color_format = color_format_to_type(sqp_img.color_format());

    let mut img = sqp_img.into_raw();
    img.truncate(width as usize * height as usize * color_format.bytes_per_pixel() as usize);

    image::save_buffer(
        args.output,
        &img,
        width,
        height,
        color_format,
    )?;

    Ok(())
}
