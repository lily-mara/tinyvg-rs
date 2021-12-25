pub use kurbo::{Line, Point, Rect};
pub use piet::Color;

#[derive(Debug, PartialEq, Clone)]
pub struct File {
    pub header: Header,
    pub color_table: Vec<Color>,
    pub commands: Vec<Command>,
    pub trailer: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ColorEncoding {
    Rgba8888,
    Rgb565,
    RgbaF32,
}

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
pub enum Command {
    FillPolygon {
        fill_style: Style,
        polygon: Vec<Point>,
    },
    FillRectangles {
        fill_style: Style,
        rectangles: Vec<Rect>,
    },
    FillPath {
        fill_style: Style,
        path: Vec<Segment>,
    },
    DrawLines {
        line_style: Style,
        line_width: f64,
        lines: Vec<Line>,
    },
    DrawLineLoop {
        line_style: Style,
        line_width: f64,
        points: Vec<Point>,
    },
    DrawLineStrip {
        line_style: Style,
        line_width: f64,
        points: Vec<Point>,
    },
    DrawLinePath {
        line_style: Style,
        line_width: f64,
        path: Vec<Segment>,
    },
    OutlineFillPolygon {
        fill_style: Style,
        line_style: Style,
        line_width: f64,
        points: Vec<Point>,
    },
    OutlineFillRectangle {
        fill_style: Style,
        line_style: Style,
        line_width: f64,
        rectangles: Vec<Rect>,
    },
    OutlineFillPath {
        fill_style: Style,
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
pub enum Sweep {
    Left,
    Right,
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
    ArcCircle {
        large: bool,
        sweep: Sweep,
        radius: f64,
        target: Point,
    },
    ArcEllipse {
        large: bool,
        sweep: Sweep,
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CoordinateRange {
    // 16 bits
    Default,

    // 8 bits
    Reduced,

    // 32 bits
    Enhanced,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub version: u8,
    pub scale: u8,
    pub color_encoding: ColorEncoding,
    pub coordinate_range: CoordinateRange,
    pub width: u32,
    pub height: u32,
    pub color_count: u32,
}
