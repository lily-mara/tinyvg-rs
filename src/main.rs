use std::path::PathBuf;

use eyre::Result;
use structopt::StructOpt;

/// TinyVG to PNG renderer
#[derive(StructOpt)]
struct Options {
    /// Optional output path. If not specified, uses the input path with a
    /// `.png` suffix.
    #[structopt(short)]
    output: Option<PathBuf>,

    /// Input path to TinyVG binary file
    input: PathBuf,
}

fn main() -> Result<()> {
    let opts = Options::from_args();

    tinyvg::render_helper::render(opts.input, opts.output)?;

    Ok(())
}
