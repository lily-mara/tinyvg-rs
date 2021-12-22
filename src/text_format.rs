use std::{fmt::Display, fmt::Result, fmt::Write};

use crate::format::*;

impl File {
    pub fn render_text(&self, w: &mut impl std::io::Write) -> eyre::Result<()> {
        struct Writer<'a, W> {
            inner: &'a mut W,
            error: Option<std::io::Error>,
        }

        impl<'a, W> Write for Writer<'a, W>
        where
            W: std::io::Write,
        {
            fn write_str(&mut self, s: &str) -> Result {
                if self.error.is_some() {
                    return Err(std::fmt::Error);
                }

                if let Err(e) = write!(self.inner, "{}", s) {
                    self.error = Some(e);
                    return Err(std::fmt::Error);
                }

                Ok(())
            }
        }

        let mut writer = Writer {
            inner: w,
            error: None,
        };

        self.to_text(&mut writer, 0)?;

        if let Some(e) = writer.error {
            return Err(e.into());
        }

        Ok(())
    }
}

trait ToTextFormat: Sized {
    fn to_text(&self, w: &mut impl Write, indent: usize) -> Result;

    fn display<'a>(&'a self) -> Wrap<'a, Self> {
        Wrap {
            inner: self,
            indent: 0,
        }
    }

    fn indent<'a>(&'a self, indent: usize) -> Wrap<'a, Self> {
        Wrap {
            inner: self,
            indent,
        }
    }
}

struct Indent(usize);

impl Display for Indent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.0 {
            write!(f, "  ")?;
        }

        Ok(())
    }
}

struct Wrap<'a, T> {
    indent: usize,
    inner: &'a T,
}

impl<'a, T> Display for Wrap<'a, T>
where
    T: ToTextFormat,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner.to_text(f, self.indent)?;

        Ok(())
    }
}

impl ToTextFormat for File {
    fn to_text(&self, w: &mut impl Write, indent: usize) -> Result {
        writeln!(w, "(tvg {}", self.header.version)?;

        self.header.to_text(w, indent + 1)?;
        NotNewlineSeparated(&self.color_table).to_text(w, indent + 1)?;
        NewlineSeparated(&self.commands).to_text(w, indent + 1)?;

        writeln!(w, ")")?;

        Ok(())
    }
}

impl ToTextFormat for Header {
    fn to_text(&self, w: &mut impl Write, indent: usize) -> Result {
        let color_format = match self.color_encoding {
            ColorEncoding::Rgb565 => "rgb565",
            ColorEncoding::Rgba8888 => "u8888",
            ColorEncoding::RgbaF32 => "rgb32",
        };

        let precision = match self.coordinate_range {
            CoordinateRange::Default => "default",
            CoordinateRange::Enhanced => "enhanced",
            CoordinateRange::Reduced => "reduced",
        };

        let mut scale = 1u32;
        for _ in 0..self.scale {
            scale *= 2;
        }

        writeln!(
            w,
            "{}({width} {height} 1/{scale} {format} {precision})",
            Indent(indent),
            width = self.width,
            height = self.height,
            scale = scale,
            format = color_format,
            precision = precision,
        )?;

        Ok(())
    }
}

struct NotNewlineSeparated<'a, T>(&'a T);

impl<'a, T> ToTextFormat for NotNewlineSeparated<'a, Vec<T>>
where
    T: ToTextFormat,
{
    fn to_text(&self, w: &mut impl Write, indent: usize) -> Result {
        writeln!(w, "{}(", Indent(indent))?;

        for c in self.0 {
            writeln!(w, "{}({})", Indent(indent + 1), c.indent(indent + 2))?;
        }

        writeln!(w, "{})", Indent(indent))?;

        Ok(())
    }
}

struct NewlineSeparated<'a, T>(&'a T);

impl<'a, T> ToTextFormat for NewlineSeparated<'a, Vec<T>>
where
    T: ToTextFormat,
{
    fn to_text(&self, w: &mut impl Write, indent: usize) -> Result {
        writeln!(w, "{}(", Indent(indent))?;

        for c in self.0 {
            writeln!(
                w,
                "{}(\n{}{})",
                Indent(indent + 1),
                c.indent(indent + 1),
                Indent(indent + 1)
            )?;
        }

        writeln!(w, "{})", Indent(indent))?;

        Ok(())
    }
}

struct NewlineSeparatedNoExtraParens<'a, T>(&'a T);

impl<'a, T> ToTextFormat for NewlineSeparatedNoExtraParens<'a, Vec<T>>
where
    T: ToTextFormat,
{
    fn to_text(&self, w: &mut impl Write, indent: usize) -> Result {
        writeln!(w, "{}(", Indent(indent))?;

        for c in self.0 {
            write!(w, "{}", c.indent(indent + 1))?;
        }

        writeln!(w, "{})", Indent(indent))?;

        Ok(())
    }
}

impl ToTextFormat for Color {
    fn to_text(&self, w: &mut impl Write, _indent: usize) -> Result {
        write!(w, "{:.3} {:.3} {:.3}", self.red, self.green, self.blue)?;
        if self.alpha != 1.0 {
            write!(w, " {:.3}", self.alpha)?;
        }

        Ok(())
    }
}
impl ToTextFormat for Command {
    fn to_text(&self, w: &mut impl Write, indent: usize) -> Result {
        match self {
            Command::FillPolygon {
                fill_style,
                polygon,
            } => {
                writeln!(
                    w,
                    "{}fill_polygon\n{}",
                    Indent(indent + 1),
                    fill_style.indent(indent + 1)
                )?;
                NewlineSeparatedNoExtraParens(polygon).to_text(w, indent + 1)?;
            }
            Command::FillRectangles {
                fill_style,
                rectangles,
            } => {
                writeln!(
                    w,
                    "{}fill_rectangles\n{}",
                    Indent(indent + 1),
                    fill_style.indent(indent + 1)
                )?;
                NotNewlineSeparated(rectangles).to_text(w, indent + 1)?;
            }
            Command::FillPath { fill_style, path } => {
                writeln!(
                    w,
                    "{}fill_path\n{}",
                    Indent(indent + 1),
                    fill_style.indent(indent + 1)
                )?;

                NewlineSeparatedNoExtraParens(path).to_text(w, indent + 1)?;
            }
            Command::DrawLines {
                line_style,
                line_width,
                lines,
            } => {
                writeln!(
                    w,
                    "{indent}draw_lines\n{line_style}\n{indent}{line_width}",
                    indent = Indent(indent + 1),
                    line_style = line_style.indent(indent + 1),
                    line_width = line_width,
                )?;
            }
            Command::DrawLineLoop {
                line_style,
                line_width,
                points,
            } => {}
            Command::DrawLineStrip {
                line_style,
                line_width,
                points,
            } => {}
            Command::DrawLinePath {
                line_style,
                line_width,
                path,
            } => {}
            Command::OutlineFillPolygon {
                fill_style,
                line_style,
                line_width,
                points,
            } => {}
            Command::OutlineFillRectangle {
                fill_style,
                line_style,
                line_width,
                rectangles,
            } => {
                writeln!(
                    w,
                    "{indent}outline_fill_rectangles\n{fill_style}\n{line_style}\n{indent}{line_width}",
                    indent = Indent(indent + 1),
                    fill_style = fill_style.indent(indent + 1),
                    line_style = line_style.indent(indent + 1),
                    line_width=line_width,
                )?;
                NotNewlineSeparated(rectangles).to_text(w, indent + 1)?;
            }
            Command::OutlineFillPath {
                fill_style,
                line_style,
                line_width,
                path,
            } => {}
        }

        Ok(())
    }
}

impl ToTextFormat for Segment {
    fn to_text(&self, w: &mut impl Write, indent: usize) -> Result {
        write!(w, "{}(", Indent(indent))?;
        self.start.to_text(w, indent)?;
        writeln!(w, ")")?;

        NotNewlineSeparated(&self.commands).to_text(w, indent)?;

        Ok(())
    }
}

impl ToTextFormat for Rectangle {
    fn to_text(&self, w: &mut impl Write, _indent: usize) -> Result {
        write!(w, "{} {} {} {}", self.x, self.y, self.width, self.height)?;

        Ok(())
    }
}

impl ToTextFormat for SegmentCommand {
    fn to_text(&self, w: &mut impl Write, indent: usize) -> Result {
        match &self.kind {
            SegmentCommandKind::Line { end } => {
                write!(
                    w,
                    "line {} {}",
                    self.line_width.indent(indent),
                    end.indent(indent)
                )?;
            }
            SegmentCommandKind::VerticalLine { y } => {
                write!(w, "vert {} {}", self.line_width.indent(indent), y)?;
            }
            SegmentCommandKind::HorizontalLine { x } => {
                write!(w, "horiz {} {}", self.line_width.indent(indent), x)?;
            }
            SegmentCommandKind::CubicBezier {
                control_0,
                control_1,
                point_1,
            } => {
                write!(
                    w,
                    "bezier {} ({}) ({}) ({})",
                    self.line_width.indent(indent),
                    control_0.indent(indent),
                    control_1.indent(indent),
                    point_1.indent(indent)
                )?;
            }
            SegmentCommandKind::ArcCircle {
                large,
                sweep,
                radius,
                target,
            } => {
                write!(
                    w,
                    "arc_circle {} {} {} {} ({})",
                    self.line_width.indent(indent),
                    radius,
                    large,
                    match sweep {
                        Sweep::Left => "false",
                        Sweep::Right => "true",
                    },
                    target.indent(indent),
                )?;
            }
            SegmentCommandKind::ArcEllipse {
                large,
                sweep,
                radius_x,
                radius_y,
                rotation,
                target,
            } => {
                write!(
                    w,
                    "arc_ellipse {} {} {} {} {} {} ({})",
                    self.line_width.display(),
                    radius_x,
                    radius_y,
                    rotation,
                    large,
                    sweep.display(),
                    target.display(),
                )?;
            }
            SegmentCommandKind::ClosePath => {
                write!(w, "close {}", self.line_width.display())?;
            }
            SegmentCommandKind::QuadraticBezier { control, point_1 } => {
                write!(
                    w,
                    "quadratic_bezier {} ({}) ({})",
                    self.line_width.display(),
                    control.display(),
                    point_1.display()
                )?;
            }
        }

        Ok(())
    }
}

impl ToTextFormat for Sweep {
    fn to_text(&self, w: &mut impl Write, _indent: usize) -> Result {
        write!(
            w,
            "{}",
            match self {
                Sweep::Left => "false",
                Sweep::Right => "true",
            }
        )?;

        Ok(())
    }
}

impl ToTextFormat for Option<f32> {
    fn to_text(&self, w: &mut impl Write, _indent: usize) -> Result {
        match self {
            Some(x) => write!(w, "{}", x)?,
            None => write!(w, "-")?,
        }

        Ok(())
    }
}

impl ToTextFormat for Point {
    fn to_text(&self, w: &mut impl Write, _indent: usize) -> Result {
        write!(w, "{} {}", self.x, self.y)?;

        Ok(())
    }
}

impl ToTextFormat for Style {
    fn to_text(&self, w: &mut impl Write, indent: usize) -> Result {
        write!(w, "{}(", Indent(indent))?;
        match self {
            Style::FlatColor { color_index } => write!(w, "flat {}", color_index)?,

            Style::LinearGradient {
                point_0,
                point_1,
                color_index_0,
                color_index_1,
            } => {
                write!(
                    w,
                    "linear ({}) ({}) {} {}",
                    point_0.display(),
                    point_1.display(),
                    color_index_0,
                    color_index_1
                )?;
            }

            Style::RadialGradient {
                point_0,
                point_1,
                color_index_0,
                color_index_1,
            } => {
                write!(
                    w,
                    "radial ({}) ({}) {} {}",
                    point_0.display(),
                    point_1.display(),
                    color_index_0,
                    color_index_1
                )?;
            }
        }
        write!(w, ")")?;
        Ok(())
    }
}
