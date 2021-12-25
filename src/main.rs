use std::path::PathBuf;

use eyre::Result;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(short)]
    output: Option<PathBuf>,

    input: PathBuf,
}

fn main() -> Result<()> {
    let opts = Options::from_args();

    tinyvg::render_helper::render(opts.input, opts.output)?;

    Ok(())
}
