use std::{fs::File, io::Read};

use tiny_vg::parser::parse_file;

fn main() -> eyre::Result<()> {
    let path = std::env::args().nth(1).unwrap();

    let mut file = File::open(path)?;
    let mut input = Vec::new();

    file.read_to_end(&mut input)?;

    let (_, file) = parse_file(&input).unwrap();

    println!("{:#?}", file);

    Ok(())
}
