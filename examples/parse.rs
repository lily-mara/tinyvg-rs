use std::fs::File;

use tiny_vg::parser::Parser;

fn main() -> eyre::Result<()> {
    let path = std::env::args().nth(1).unwrap();

    let parser = Parser::new(File::open(path)?);

    let file = parser.parse()?;

    println!("{:#?}", file);

    Ok(())
}
