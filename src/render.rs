use cairo::{Format, ImageSurface};
use eyre::{Context, Result};
use kurbo::{BezPath, Vec2};
use piet::kurbo::{Point, Size};
use piet::{Color, FixedLinearGradient, FixedRadialGradient, GradientStop, RenderContext};
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

impl crate::format::File {
    fn brush<R>(&self, rc: &mut R, style: &Style) -> Result<R::Brush>
    where
        R: RenderContext,
    {
        let brush = match style {
            Style::FlatColor { color_index } => {
                rc.solid_brush(self.color_table[*color_index].clone())
            }
            Style::LinearGradient {
                point_0,
                point_1,
                color_index_0,
                color_index_1,
            } => rc
                .gradient(FixedLinearGradient {
                    start: *point_0,
                    end: *point_1,
                    stops: vec![
                        GradientStop {
                            pos: 0.0,
                            color: self.color_table[*color_index_0].clone(),
                        },
                        GradientStop {
                            pos: 1.0,
                            color: self.color_table[*color_index_1].clone(),
                        },
                    ],
                })
                .map_err(|e| eyre::eyre!("{}", e))?,
            Style::RadialGradient {
                point_0,
                point_1,
                color_index_0,
                color_index_1,
            } => rc
                .gradient(FixedRadialGradient {
                    center: *point_0,
                    origin_offset: Vec2 { x: 0.0, y: 0.0 },
                    radius: point_0.distance(*point_1),
                    stops: vec![
                        GradientStop {
                            pos: 0.0,
                            color: self.color_table[*color_index_0].clone(),
                        },
                        GradientStop {
                            pos: 1.0,
                            color: self.color_table[*color_index_1].clone(),
                        },
                    ],
                })
                .map_err(|e| eyre::eyre!("{}", e))?,
        };

        Ok(brush)
    }
}

fn draw_path<R>(rc: &mut R, fill: R::Brush, line: R::Brush, line_width: f64, path: &[Segment])
where
    R: RenderContext,
{
    let mut bezier = BezPath::new();

    for Segment { start, commands } in path {
        let mut pen = *start;

        bezier.move_to(pen);

        for SegmentCommand {
            kind,
            line_width: _,
        } in commands
        {
            // TODO: line width

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
                SegmentCommandKind::HorizontalLine { x } => {
                    let end = Point { x: *x, y: pen.y };

                    bezier.line_to(end);
                    pen = end;
                }
                SegmentCommandKind::ArcCircle { .. } => {
                    // TODO: circle
                }
                SegmentCommandKind::ArcEllipse { .. } => {
                    // TODO: ellipse
                }
                SegmentCommandKind::ClosePath => {
                    bezier.line_to(*start);
                    pen = *start;
                }
                SegmentCommandKind::QuadraticBezier { control, point_1 } => {
                    bezier.quad_to(*control, *point_1);
                    pen = *point_1;
                }
            }
        }
    }

    rc.fill(&bezier, &fill);
    rc.stroke(&bezier, &line, line_width);
}

fn draw(f: &crate::format::File, rc: &mut impl RenderContext) -> Result<()> {
    // rc.clear(None, Color::WHITE);
    for cmd in &f.commands {
        match cmd {
            Command::FillPath { fill_style, path } => {
                let fill = f.brush(rc, fill_style)?;
                let line = rc.solid_brush(Color::rgba(0.0, 0.0, 0.0, 0.0));

                draw_path(rc, fill, line, 0.0, path);
            }
            Command::FillRectangles {
                fill_style,
                rectangles,
            } => {
                let brush = f.brush(rc, fill_style)?;

                for rect in rectangles {
                    rc.fill(rect, &brush);
                }
            }
            Command::OutlineFillRectangle {
                fill_style,
                line_style,
                line_width,
                rectangles,
            } => {
                let fill = f.brush(rc, fill_style)?;
                let line = f.brush(rc, line_style)?;

                for rect in rectangles {
                    rc.stroke(rect, &line, *line_width);

                    rc.fill(rect, &fill);
                }
            }
            Command::FillPolygon {
                fill_style,
                polygon,
            } => {
                let brush = f.brush(rc, fill_style)?;

                let mut bez = BezPath::new();
                bez.move_to(polygon[0]);

                for point in polygon {
                    bez.line_to(*point);
                }

                rc.fill(bez, &brush);
            }
            Command::DrawLines {
                line_style,
                line_width,
                lines,
            } => {
                let brush = f.brush(rc, line_style)?;

                for line in lines {
                    rc.stroke(line, &brush, *line_width);
                }
            }
            Command::OutlineFillPolygon {
                fill_style,
                line_style,
                line_width,
                points,
            } => {
                let fill = f.brush(rc, &fill_style)?;
                let line = f.brush(rc, &line_style)?;

                let mut bez = BezPath::new();
                bez.move_to(points[0]);

                for point in points {
                    bez.line_to(*point);
                }

                rc.fill(&bez, &fill);
                rc.stroke(&bez, &line, *line_width);
            }
            Command::OutlineFillPath {
                fill_style,
                line_style,
                line_width,
                path,
            } => {
                let fill = f.brush(rc, fill_style)?;
                let line = f.brush(rc, line_style)?;

                draw_path(rc, fill, line, *line_width, path);
            }
            _ => {}
        }
    }

    Ok(())
}
