//! SQP (SQuishy Picture Format) is an image format. It can be used to store
//! image data in lossless or lossy compressed form, while remaining relatively
//! simple.

mod compression {
    pub mod dct;
    pub mod lossless;
}
mod binio;
mod operations;

pub mod picture;
pub mod header;
