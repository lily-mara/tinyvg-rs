# tinyvg-rs

This is a Rust implementation of the [TinyVG](https://tinyvg.tech) image format.
It provides an executable that renders PNG images from TinyVG input files, and a
library that can render PNG images or any format supported by
`piet::RenderContext`.

# Executable

## Installation

```
$ cargo install tinyvg
```

## Usage

```
tinyvg 0.1.0
TinyVG to PNG renderer

USAGE:
    tinyvg [OPTIONS] <input>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o <output>        Optional output path. If not specified, uses the input path with a `.png` suffix

ARGS:
    <input>    Input path to TinyVG binary file
```

# Library Usage

```rust
use tinyvg::Parser;
use std::fs::File;

fn main() -> eyre::Result<()> {
    // Build a parser from a `std::io::Read`. Here a file is used, but any type
    // that implements `Read` can be used.
    let parser = Parser::new(File::open("data/shield.tvg")?);

    let image = parser.parse()?;

    let mut out = File::create("out.png")?;

    // Render the image to a PNG file. Here a file is used, but any type
    // that implements `std::io::Write` can be used.
    image.render_png(&mut out)?;

    Ok(())
}
```
