#[derive(Debug, PartialEq, Clone)]
pub struct File {
    pub header: Header,
    pub color_table: Vec<Color>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ColorEncoding {
    Rgba8888,
    Rgb565,
    RgbaF32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CoordinateRange {
    // 8 bits
    Default,

    // 16 bits
    Reduced,

    // 32 bits
    Enhanced,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
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
