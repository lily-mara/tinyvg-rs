use std::{fs::File, path::PathBuf};

use eyre::Context;
use tiny_vg::{parser::Parser, render};

fn main() -> eyre::Result<()> {
    let path = PathBuf::from(std::env::args().nth(1).unwrap());

    let parser = Parser::new(File::open(&path)?);

    let image = parser.parse()?;

    let mut new_path = path.clone();
    new_path.set_extension("png");

    let mut file = File::create(&new_path).wrap_err("failed to create output file")?;

    render(&image, &mut file)?;

    Ok(())
}
