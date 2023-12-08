//! Helper function that can render a TinyVG image using only the path to the input file

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use crate::decode::Decoder;
use eyre::{Context, Result};

/// Render a TinyVG file using input and output path. If the output path is not
/// specified, it will be automatically determined by adding the `.png` suffix
/// to the input path
///
/// ```
/// # use tinyvg::render_helper::render;
/// render(
///   "data/shield.tvg",
///   Some("data/shield-render.png".into())
/// ).unwrap();
/// ```
#[cfg(feature = "render-png")]
pub fn render(in_path: impl AsRef<Path>, out_path: Option<PathBuf>) -> Result<()> {
    let mut decoder = Decoder::new(BufReader::new(File::open(&in_path)?));

    let mut image = decoder.decode_header()?;

    let result = decoder.decode_commands(&mut image);

    let out_path = out_path.unwrap_or_else(|| {
        let mut out_path = in_path.as_ref().to_owned();
        out_path.set_extension("png");

        out_path
    });

    let mut file =
        BufWriter::new(File::create(out_path).wrap_err("failed to create output file")?);
    image.render_png(&mut file)?;

    result?;

    Ok(())
}
