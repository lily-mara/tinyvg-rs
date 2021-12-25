# tinyvg-rs

This is a Rust implementation of the [TinyVG](https://tinyvg.tech) image format.
It provides an executable that renders PNG images from TinyVG input files, and a
library that can render PNG images or any format supported by
`piet::RenderContext`.

# Dependencies

All dependencies but one are managed by cargo. This program/library does depend
on cairo for rendering PNGs. You should be able to install cairo using your OS
package manager.

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

# Development

## Testing

There are some doctests which validate that certain files can be parsed and
rendered without errors, but there currently isn't much in the way of automated
testing. I wasn't sure how to effectively write equality tests without writing a
massive amount of code. There is an example program which will crawl the `data`
directory and render all `.tvg` files into `.png` files with the same names.
This can be used to validate rendering behavior for example images.

```
$ cargo run --example render-all
path                 render time
data/app_icon.tvg    78.94075ms
data/chart.tvg       14.194083ms
data/comic.tvg       23.534791ms
data/everything.tvg  21.251875ms
data/flowchart.tvg   5.075ms
data/shield.tvg      597.916µs
data/tiger.tvg       35.942166ms

$ open data/tiger.png
```

There is also a criterion benchmarking suite which tests decoding and rendering.

```
$ cargo bench
TinyVG/decode/tiger.tvg time:   [135.98 us 136.65 us 137.34 us]
TinyVG/render/tiger.tvg time:   [27.541 ms 27.629 ms 27.744 ms]
```
