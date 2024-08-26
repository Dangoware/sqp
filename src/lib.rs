//! SQP (**SQ**uishy **P**icture Format) is an image format. It can be used to store
//! image data in lossless or lossy compressed form. It is designed to be
//! relatively simple compared to other more standard formats.
//!
//! This image format is mainly for experimentation and learning about
//! compression. While it can be used, there are no guarantees about stability,
//! breaking changes, or features.
//!
//! If you're looking for an image format to use, you might want to consider
//! using a more standard one such as those supported by the
//! [image crate](https://docs.rs/image/latest/image/).
//!
//! # Example
//! ## Creating and writing an SQP
//! ```no_run
//! use sqp::{SquishyPicture, ColorFormat};
//!
//! let width = 2;
//! let height = 2;
//! let bitmap = vec![
//!     0xFF, 0xFF, 0xFF, 0xFF,
//!     0x00, 0x80, 0x00, 0x80,
//!     0xFF, 0xFF, 0xFF, 0xFF,
//!     0x00, 0x80, 0x00, 0x80,
//! ];
//!
//! // Create a 2×2 image in memory. Nothing is compressed or encoded
//! // at this point.
//! let sqp_image = SquishyPicture::from_raw_lossless(
//!     width,
//!     height,
//!     ColorFormat::Rgba8,
//!     bitmap
//! );
//!
//! // Write it out to a file. This performs compression and encoding.
//! sqp_image.save("my_image.sqp").expect("Could not save the image");
//! ```
//!
//! ## Reading an SQP from a file.
//! ```no_run
//! use std::fs::File;
//! use sqp::SquishyPicture;
//!
//! // Load it directly with the `open` function...
//! let image = sqp::open("my_image.sqp").expect("Could not open file");
//!
//! // ...or from something implementing Read.
//! let input_file = File::open("my_image.sqp").expect("Could not open image file");
//! let image2 = SquishyPicture::decode(&input_file);
//! ```

mod compression {
    pub mod dct;
    pub mod lossless;
}
mod binio;
mod operations;

pub mod picture;
pub mod header;

// ----------------------- //
// INLINED USEFUL FEATURES //
// ----------------------- //
#[doc(inline)]
pub use picture::SquishyPicture;

#[doc(inline)]
pub use picture::open;

#[doc(inline)]
pub use header::ColorFormat;

#[doc(inline)]
pub use header::CompressionType;
