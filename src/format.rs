#![allow(missing_docs)]

//! In-memory representation of a TinyVG file

pub use kurbo::{Line, Point, Rect};
pub use piet::Color;

/// A single TinyVG file
#[derive(Debug, PartialEq, Clone)]
pub struct Image {
    /// Image header
    pub header: Header,

    /// The colors used in this image
    pub color_table: Vec<Color>,

    /// TinyVG commands required to render this image
    pub commands: Vec<Command>,

    /// Remaining data after the TinyVG image ended. Can be used for arbitrary
    /// metadata, it is not defined by the spec
    pub trailer: Vec<u8>,
}

/// Types of color values. Only useful for encoding/decoding TinyVG binary
/// format.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ColorEncoding {
    /// RGBA color with 8 bits per channel
    Rgba8888,

    /// RGB color with 5 bits on red channel, 6 bits on green channel, 5 bits on blue channel
    Rgb565,

    /// RGBA color made up of 4 f32 values
    RgbaF32,
}

/// Styles refer to the color or gradients for a line or filling
#[derive(Debug, PartialEq, Clone)]
pub enum Style {
    FlatColor {
        color_index: usize,
    },
    LinearGradient {
        point_0: Point,
        point_1: Point,
        color_index_0: usize,
        color_index_1: usize,
    },
    RadialGradient {
        point_0: Point,
        point_1: Point,
        color_index_0: usize,
        color_index_1: usize,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct OutlineStyle {
    pub line_width: f64,
    pub line_style: Style,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    FillPolygon {
        fill_style: Style,
        polygon: Vec<Point>,
        outline: Option<OutlineStyle>,
    },
    FillRectangles {
        fill_style: Style,
        rectangles: Vec<Rect>,
        outline: Option<OutlineStyle>,
    },
    FillPath {
        fill_style: Style,
        path: Vec<Segment>,
        outline: Option<OutlineStyle>,
    },
    DrawLines {
        line_style: Style,
        line_width: f64,
        lines: Vec<Line>,
    },
    DrawLineLoop {
        line_style: Style,
        line_width: f64,
        close_path: bool,
        points: Vec<Point>,
    },
    DrawLinePath {
        line_style: Style,
        line_width: f64,
        path: Vec<Segment>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Segment {
    pub start: Point,
    pub commands: Vec<SegmentCommand>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SegmentCommand {
    pub kind: SegmentCommandKind,
    pub line_width: Option<f64>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SegmentCommandKind {
    Line {
        end: Point,
    },
    HorizontalLine {
        x: f64,
    },
    VerticalLine {
        y: f64,
    },
    CubicBezier {
        control_0: Point,
        control_1: Point,
        point_1: Point,
    },
    ArcEllipse {
        large: bool,
        sweep: bool,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
        target: Point,
    },
    ClosePath,
    QuadraticBezier {
        control: Point,
        point_1: Point,
    },
}

/// Width of certain coordinate values. Only useful for encoding/decoding TinyVG
/// binary format.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CoordinateRange {
    /// 16 bits per coordinate value
    Default,

    /// 8 bits per coordinate value
    Reduced,

    /// 32 bits per coordinate value
    Enhanced,
}

/// Image header for TinyVG image. Mostly useful for binary encoding/decoding.
#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    /// Version must be 1
    pub version: u8,

    /// Value used when reading `Unit` values in the parser. y
    pub scale: u8,

    /// Binary encoding used for color values. Only useful for encoding/decoding
    /// TinyVG binary format.
    pub color_encoding: ColorEncoding,

    /// Width of binary values for certain fields. Only useful for
    /// encoding/decoding TinyVG binary format.
    pub coordinate_range: CoordinateRange,

    /// Width in pixels
    pub width: u32,

    /// Height in pixels
    pub height: u32,

    /// Number of colors in this image. Equivalent to `file.color_table.len()`
    pub color_count: u32,
}
