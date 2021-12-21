use nom::{
    bytes::complete::{tag, take_while},
    combinator::map,
    error::ErrorKind,
    multi::count,
    number::complete::{le_f32, le_u16, le_u32, le_u8},
    sequence::tuple,
    IResult,
};

use crate::format::{Color, ColorEncoding, CoordinateRange, File, Header};

fn magic_number(input: &[u8]) -> IResult<&[u8], ()> {
    tag([0x72, 0x56])(input).map(|(rest, _)| (rest, ()))
}

fn version(input: &[u8]) -> IResult<&[u8], u8> {
    le_u8(input)
}

struct ScaleProperties {
    scale: u8,
    color_encoding: ColorEncoding,
    coordinate_range: CoordinateRange,
}

fn scale_properties(input: &[u8]) -> IResult<&[u8], ScaleProperties> {
    let (rest, x) = le_u8(input)?;
    let scale = (x & 0xF0) >> 4;
    let color_encoding = (x & 0b0000_1100) >> 2;
    let coordinate_range = x & 0b0000_0011;

    let coordinate_range = match coordinate_range {
        0 => CoordinateRange::Default,
        1 => CoordinateRange::Reduced,
        2 => CoordinateRange::Enhanced,
        _ => {
            // TODO: make a better error message here
            return Err(nom::Err::Failure(nom::error::Error::new(
                input,
                ErrorKind::Verify,
            )));
        }
    };

    let color_encoding = match color_encoding {
        0 => ColorEncoding::Rgba8888,
        1 => ColorEncoding::Rgb565,
        2 => ColorEncoding::RgbaF32,
        3 => {
            // TODO: make a better error message here - custom is unsupported
            return Err(nom::Err::Failure(nom::error::Error::new(
                input,
                ErrorKind::Verify,
            )));
        }
        _ => {
            // TODO: make a better error message here
            return Err(nom::Err::Failure(nom::error::Error::new(
                input,
                ErrorKind::Verify,
            )));
        }
    };

    Ok((
        rest,
        ScaleProperties {
            scale,
            color_encoding,
            coordinate_range,
        },
    ))
}

fn width_height(coordinate_range: CoordinateRange) -> impl Fn(&[u8]) -> IResult<&[u8], u32> {
    move |input| match coordinate_range {
        CoordinateRange::Default => map(le_u8, |x| x as u32)(input),
        CoordinateRange::Reduced => map(le_u16, |x| x as u32)(input),
        CoordinateRange::Enhanced => map(le_u32, |x| x as u32)(input),
    }
}

fn var_uint(input: &[u8]) -> IResult<&[u8], u32> {
    map(take_while(|b| (b & 0x80) == 0), |bytes: &[u8]| {
        let mut result = 0u32;

        for (i, b) in bytes.iter().enumerate() {
            let b = *b as u32;
            result |= (b & 0x7F) << (7 * i);
        }

        result
    })(input)
}

fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (rest, ((), version, scale_properties)) =
        tuple((magic_number, version, scale_properties))(input)?;

    let (rest, (width, height)) = tuple((
        width_height(scale_properties.coordinate_range),
        width_height(scale_properties.coordinate_range),
    ))(rest)?;

    let (rest, color_count) = var_uint(rest)?;

    Ok((
        rest,
        Header {
            version,
            scale: scale_properties.scale,
            color_encoding: scale_properties.color_encoding,
            coordinate_range: scale_properties.coordinate_range,
            width,
            height,
            color_count,
        },
    ))
}

fn parse_color_table(
    color_encoding: ColorEncoding,
    color_count: u32,
) -> impl Fn(&[u8]) -> IResult<&[u8], Vec<Color>> {
    move |input| match color_encoding {
        ColorEncoding::Rgba8888 => count(color_8888, color_count as usize)(input),
        ColorEncoding::RgbaF32 => count(color_f32, color_count as usize)(input),
        ColorEncoding::Rgb565 => count(color_565, color_count as usize)(input),
    }
}

fn color_8888(input: &[u8]) -> IResult<&[u8], Color> {
    map(
        tuple((le_u8, le_u8, le_u8, le_u8)),
        |(red, green, blue, alpha)| Color {
            red: red as f32 / 255.0,
            green: green as f32 / 255.0,
            blue: blue as f32 / 255.0,
            alpha: alpha as f32 / 255.0,
        },
    )(input)
}

fn color_f32(input: &[u8]) -> IResult<&[u8], Color> {
    map(
        tuple((le_f32, le_f32, le_f32, le_f32)),
        |(red, green, blue, alpha)| Color {
            red,
            green,
            blue,
            alpha,
        },
    )(input)
}

fn color_565(input: &[u8]) -> IResult<&[u8], Color> {
    map(le_u16, |rgb| Color {
        red: (((rgb & 0x001F) >> 0) as f32) / 31.0,
        green: (((rgb & 0x07E0) >> 5) as f32) / 63.0,
        blue: (((rgb & 0xF800) >> 11) as f32) / 31.0,
        alpha: 1.0,
    })(input)
}

pub fn parse_file(input: &[u8]) -> IResult<&[u8], File> {
    let (rest, header) = parse_header(input)?;
    let (rest, color_table) = parse_color_table(header.color_encoding, header.color_count)(rest)?;

    Ok((
        rest,
        File {
            header,
            color_table,
        },
    ))
}
