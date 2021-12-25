use cairo::{Format, ImageSurface};
use eyre::{Context, Result};
use kurbo::BezPath;
use piet::kurbo::{Point, Size};
use piet::RenderContext;
use piet_cairo::CairoRenderContext;

use crate::format::{Command, Segment, SegmentCommand, SegmentCommandKind, Style};

pub fn render(f: &crate::format::File, writer: &mut impl std::io::Write) -> Result<()> {
    let size = Size {
        width: f.header.width as f64,
        height: f.header.height as f64,
    };

    let surface = ImageSurface::create(Format::ARgb32, size.width as i32, size.height as i32)
        .wrap_err("failed to create cairo surface")?;
    let cr = cairo::Context::new(&surface).unwrap();

    {
        let mut piet_context = CairoRenderContext::new(&cr);

        draw(f, &mut piet_context).wrap_err("failed to draw tinyvg file")?;

        piet_context
            .finish()
            .map_err(|e| eyre::eyre!("{}", e))
            .wrap_err("failed to finalize piet context")?;
    }

    surface.flush();
    surface.write_to_png(writer)?;

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
                        brush = rc.solid_brush(f.color_table[*color_index].clone());
                    }
                    _ => {}
                }

                let mut bezier = BezPath::new();

                for Segment { start, commands } in path {
                    let mut pen = *start;

                    bezier.move_to(pen);

                    // let mut width = 1.0f64;

                    for SegmentCommand {
                        kind,
                        line_width: _,
                    } in commands
                    {
                        // if let Some(new_width) = line_width {
                        // width = *new_width as f64;
                        // }

                        match kind {
                            SegmentCommandKind::Line { end } => {
                                pen = *end;
                                bezier.line_to(*end);
                            }
                            SegmentCommandKind::VerticalLine { y } => {
                                let end = Point { x: pen.x, y: *y };

                                bezier.line_to(end);
                                pen = end;
                            }
                            SegmentCommandKind::CubicBezier {
                                control_0,
                                control_1,
                                point_1,
                            } => {
                                bezier.curve_to(*control_0, *control_1, *point_1);
                                pen = *point_1;
                            }
                            _ => {}
                        }
                    }
                }

                rc.fill(bezier, &brush);
            }
            _ => {}
        }
    }

    Ok(())
}
