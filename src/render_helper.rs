use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::{fs::File, path::PathBuf};

use crate::parser::Parser;
use eyre::{Context, Result};

pub fn render(in_path: impl AsRef<Path>, out_path: Option<PathBuf>) -> Result<()> {
    let mut parser = Parser::new(BufReader::new(File::open(&in_path)?));

    let mut image = parser.parse_header()?;

    let result = parser.parse_commands(&mut image);

    let out_path = out_path.unwrap_or_else(|| {
        let mut out_path = in_path.as_ref().to_owned();
        out_path.set_extension("png");

        out_path
    });

    let mut file =
        BufWriter::new(File::create(&out_path).wrap_err("failed to create output file")?);
    image.render_png(&mut file)?;

    result?;

    Ok(())
}
