#![warn(missing_docs)]
//! Decoder and renderer for the TinyVG vector graphics format

pub mod decode;
pub mod format;
mod render;

pub mod render_helper;

pub use decode::Decoder;
pub use format::Image;
