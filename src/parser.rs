use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use eyre::{bail, ensure, Result};

use crate::format::{Color, ColorEncoding, CoordinateRange, File, Header};

pub struct Parser<R> {
    reader: R,
    coordinate_range: CoordinateRange,
    color_count: u32,
    color_encoding: ColorEncoding,
}

impl<R> Parser<R>
where
    R: Read,
{
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            coordinate_range: CoordinateRange::Default,
            color_count: 0,
            color_encoding: ColorEncoding::Rgb565,
        }
    }

    fn magic_number(&mut self) -> Result<()> {
        let b0 = self.reader.read_u8()?;
        let b1 = self.reader.read_u8()?;

        ensure!(
            b0 == 0x72 && b1 == 0x56,
            "tinyvg file must begin with magic number 0x72 0x56, found {:x} {:x}",
            b0,
            b1
        );

        Ok(())
    }

    fn version(&mut self) -> Result<u8> {
        let version = self.reader.read_u8()?;

        Ok(version)
    }

    fn scale_properties(&mut self) -> Result<ScaleProperties> {
        let x = self.reader.read_u8()?;
        let scale = (x & 0xF0) >> 4;
        let color_encoding = (x & 0b0000_1100) >> 2;
        let coordinate_range = x & 0b0000_0011;

        let coordinate_range = match coordinate_range {
            0 => CoordinateRange::Default,
            1 => CoordinateRange::Reduced,
            2 => CoordinateRange::Enhanced,
            x => {
                bail!("unrecognized coordinate type {}", x);
            }
        };

        let color_encoding = match color_encoding {
            0 => ColorEncoding::Rgba8888,
            1 => ColorEncoding::Rgb565,
            2 => ColorEncoding::RgbaF32,
            3 => {
                bail!("custom color encodings are not supported");
            }
            x => {
                bail!("unrecognized color encoding {}", x);
            }
        };

        Ok(ScaleProperties {
            scale,
            color_encoding,
            coordinate_range,
        })
    }

    fn width_height(&mut self) -> Result<u32> {
        match self.coordinate_range {
            CoordinateRange::Default => {
                let x = self.reader.read_u8()?;
                Ok(x as u32)
            }
            CoordinateRange::Reduced => {
                let x = self.reader.read_u16::<LittleEndian>()?;
                Ok(x as u32)
            }
            CoordinateRange::Enhanced => {
                let x = self.reader.read_u32::<LittleEndian>()?;
                Ok(x as u32)
            }
        }
    }

    fn var_uint(&mut self) -> Result<u32> {
        let mut result = 0u32;
        let mut count = 0;

        loop {
            let b = self.reader.read_u8()? as u32;

            result |= (b & 0x7F) << (7 * count);

            if (b & 0x80) == 0 {
                break;
            }

            count += 1;
        }

        Ok(result)
    }

    fn parse_color_table(&mut self) -> Result<Vec<Color>> {
        let mut colors = Vec::new();

        for _ in 0..self.color_count {
            colors.push(match self.color_encoding {
                ColorEncoding::Rgba8888 => self.color_8888()?,
                ColorEncoding::RgbaF32 => self.color_f32()?,
                ColorEncoding::Rgb565 => self.color_565()?,
            })
        }

        Ok(colors)
    }

    fn color_8888(&mut self) -> Result<Color> {
        let red = self.reader.read_u8()?;
        let green = self.reader.read_u8()?;
        let blue = self.reader.read_u8()?;
        let alpha = self.reader.read_u8()?;

        Ok(Color {
            red: red as f32 / 255.0,
            green: green as f32 / 255.0,
            blue: blue as f32 / 255.0,
            alpha: alpha as f32 / 255.0,
        })
    }

    fn color_f32(&mut self) -> Result<Color> {
        let red = self.reader.read_f32::<LittleEndian>()?;
        let green = self.reader.read_f32::<LittleEndian>()?;
        let blue = self.reader.read_f32::<LittleEndian>()?;
        let alpha = self.reader.read_f32::<LittleEndian>()?;

        Ok(Color {
            red,
            green,
            blue,
            alpha,
        })
    }

    fn color_565(&mut self) -> Result<Color> {
        let rgb = self.reader.read_u16::<LittleEndian>()?;

        Ok(Color {
            red: (((rgb & 0x001F) >> 0) as f32) / 31.0,
            green: (((rgb & 0x07E0) >> 5) as f32) / 63.0,
            blue: (((rgb & 0xF800) >> 11) as f32) / 31.0,
            alpha: 1.0,
        })
    }

    fn header(&mut self) -> Result<Header> {
        self.magic_number()?;
        let version = self.version()?;
        let scale_properties = self.scale_properties()?;

        self.coordinate_range = scale_properties.coordinate_range;
        let width = self.width_height()?;
        let height = self.width_height()?;

        let color_count = self.var_uint()?;

        self.color_count = color_count;
        self.color_encoding = scale_properties.color_encoding;

        Ok(Header {
            version,
            scale: scale_properties.scale,
            color_encoding: scale_properties.color_encoding,
            coordinate_range: scale_properties.coordinate_range,
            width,
            height,
            color_count,
        })
    }

    pub fn parse(mut self) -> Result<File> {
        let header = self.header()?;
        let color_table = self.parse_color_table()?;

        Ok(File {
            header,
            color_table,
            commands: Vec::new(),
            trailer: Vec::new(),
        })
    }
}

struct ScaleProperties {
    scale: u8,
    color_encoding: ColorEncoding,
    coordinate_range: CoordinateRange,
}

#[cfg(test)]
mod tests {
    use super::Parser;
    use eyre::Result;
    use std::fs::File;

    #[test]
    fn test_parser() -> Result<()> {
        let file = File::open("data/everything.tvg")?;
        let p = Parser::new(file);

        let result = p.parse()?;

        insta::assert_debug_snapshot!(result);

        Ok(())
    }
}
