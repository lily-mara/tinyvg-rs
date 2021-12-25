use std::io::Write;
use std::{fs::File, path::PathBuf};

use eyre::{Context, Result};
use tinyvg::parser::Parser;

fn main() -> Result<()> {
    for path in glob::glob("./data/*.tvg")? {
        let path = path?;
        render(path)?;
    }

    Ok(())
}

fn render(path: PathBuf) -> Result<()> {
    let mut parser = Parser::new(File::open(&path)?);

    let mut image = parser.parse_header()?;

    let result = parser.parse_commands(&mut image);

    let mut dbg_path = path.clone();
    dbg_path.set_extension("txt");
    let mut dbg_file = File::create(&dbg_path).wrap_err("failed to create output file")?;
    writeln!(&mut dbg_file, "{:#?}", image)?;

    let mut new_path = path.clone();
    new_path.set_extension("png");
    let mut file = File::create(&new_path).wrap_err("failed to create output file")?;
    image.render_png(&mut file)?;

    result?;

    eprintln!("{}", new_path.display());

    Ok(())
}
