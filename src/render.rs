use cairo::{Format, ImageSurface};
use eyre::{Context, Result};
use piet::kurbo::Size;
use piet::RenderContext;
use piet_cairo::CairoRenderContext;
use std::fs::File;

use crate::format::{Command, Point, Segment, SegmentCommand, SegmentCommandKind, Style};

pub fn render(f: &crate::format::File) -> Result<()> {
    let size = Size {
        width: f.header.width as f64,
        height: f.header.height as f64,
    };

    let surface = ImageSurface::create(Format::ARgb32, size.width as i32, size.height as i32)
        .wrap_err("failed to create cairo surface")?;
    let cr = cairo::Context::new(&surface).unwrap();
    let mut piet_context = CairoRenderContext::new(&cr);

    draw(f, &mut piet_context).wrap_err("failed to draw tinyvg file")?;

    piet_context
        .finish()
        .map_err(|e| eyre::eyre!("{}", e))
        .wrap_err("failed to finalize piet context")?;

    std::mem::drop(piet_context);

    let mut file = File::create("out.png").wrap_err("failed to create output file")?;
    surface.flush();
    surface.write_to_png(&mut file)?;

    Ok(())
}

fn draw(f: &crate::format::File, rc: &mut impl RenderContext) -> Result<()> {
    // rc.clear(None, Color::WHITE);
    for cmd in &f.commands {
        match cmd {
            Command::FillPath { fill_style, path } => {
                let mut brush = rc.solid_brush(piet::Color::rgb8(0, 0, 0));

                match fill_style {
                    Style::FlatColor { color_index } => {
                        brush = rc.solid_brush(f.get_color(*color_index));
                    }
                    _ => {}
                }

                for Segment { start, commands } in path {
                    let mut pen = *start;
                    let mut width = 1.0f64;

                    for cmd in commands {
                        if let Some(new_width) = cmd.line_width {
                            width = new_width as f64;
                        }

                        match cmd.kind {
                            SegmentCommandKind::Line { end } => {
                                let line = kurbo::Line {
                                    p0: pen.into(),
                                    p1: end.into(),
                                };
                                rc.stroke(line, &brush, width);
                                pen = end;
                            }
                            SegmentCommandKind::VerticalLine { y } => {
                                let end = Point { x: pen.x, y };
                                let line = kurbo::Line {
                                    p0: pen.into(),
                                    p1: end.into(),
                                };
                                rc.stroke(line, &brush, width);
                                pen = end;
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

impl Into<kurbo::Point> for crate::format::Point {
    fn into(self) -> kurbo::Point {
        kurbo::Point {
            x: self.x as f64,
            y: self.y as f64,
        }
    }
}

impl Into<piet::Color> for crate::format::Color {
    fn into(self) -> piet::Color {
        piet::Color::rgba(
            self.red as f64,
            self.green as f64,
            self.blue as f64,
            self.alpha as f64,
        )
    }
}

impl crate::format::File {
    fn get_color(&self, color_index: usize) -> piet::Color {
        let color = self.color_table[color_index];

        color.into()
    }
}
