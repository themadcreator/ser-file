use binrw::{BinRead, BinResult, BinWrite, VecArgs, binrw};
use image::{DynamicImage, GenericImageView, ImageBuffer};

use crate::ColorId;

#[derive(Debug, Clone)]
pub enum PixelChannels {
    Rgb,
    Luma,
}

#[binrw]
#[br(map(i32::into))]
#[bw(map(i32::from))]
#[derive(Debug, Clone)]
pub enum PixelDepth {
    U8(i32),
    U16(i32),
}

#[derive(Clone)]
pub enum Pixels {
    Rgb8(Vec<u8>),
    Rgb16(Vec<u16>),
    Luma8(Vec<u8>),
    Luma16(Vec<u16>),
}

#[derive(Clone)]
pub struct Frame((u32, u32), Pixels);

#[derive(Clone)]
pub struct FrameFormat {
    pub(crate) color: ColorId,
    pub(crate) depth: PixelDepth,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl FrameFormat {
    pub fn new(color: ColorId, depth: PixelDepth, width: u32, height: u32) -> Self {
        Self {
            color,
            depth,
            width,
            height,
        }
    }

    pub fn try_new_frame(&self, img: DynamicImage) -> Result<Frame, &'static str> {
        if img.width() != self.width || img.height() != self.height {
            return Err(
                "Incompatible image dimensions. All frames must have the same width and height.",
            );
        }

        let pixels = match ((&self.color).into(), &self.depth) {
            (PixelChannels::Luma, PixelDepth::U8(_)) => Pixels::Luma8(img.to_luma8().into_raw()),
            (PixelChannels::Luma, PixelDepth::U16(_)) => Pixels::Luma16(img.to_luma16().into_raw()),
            (PixelChannels::Rgb, PixelDepth::U8(_)) => Pixels::Rgb8(img.to_rgb8().into_raw()),
            (PixelChannels::Rgb, PixelDepth::U16(_)) => Pixels::Rgb16(img.to_rgb16().into_raw()),
        };
        Ok(Frame((self.width, self.height), pixels))
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl PixelChannels {
    fn len(&self) -> usize {
        match self {
            PixelChannels::Rgb => 3,
            PixelChannels::Luma => 1,
        }
    }
}

impl TryFrom<&DynamicImage> for FrameFormat {
    type Error = &'static str;

    fn try_from(value: &DynamicImage) -> Result<FrameFormat, Self::Error> {
        let (width, height) = value.dimensions();
        match value {
            DynamicImage::ImageRgb8(_) => Ok(FrameFormat::new(
                ColorId::RGB,
                PixelDepth::U8(8),
                width,
                height,
            )),
            DynamicImage::ImageRgb16(_) => Ok(FrameFormat::new(
                ColorId::RGB,
                PixelDepth::U16(16),
                width,
                height,
            )),
            DynamicImage::ImageLuma8(_) => Ok(FrameFormat::new(
                ColorId::MONO,
                PixelDepth::U8(8),
                width,
                height,
            )),
            DynamicImage::ImageLuma16(_) => Ok(FrameFormat::new(
                ColorId::MONO,
                PixelDepth::U16(16),
                width,
                height,
            )),
            _ => Err("Unsupported image type."),
        }
    }
}

impl TryFrom<Frame> for DynamicImage {
    type Error = &'static str;

    fn try_from(value: Frame) -> Result<Self, Self::Error> {
        let Frame((width, height), pixels) = value;
        match match pixels {
            Pixels::Rgb8(p) => ImageBuffer::<image::Rgb<u8>, Vec<u8>>::from_raw(width, height, p)
                .and_then(|buf| Some(buf.into())),
            Pixels::Rgb16(p) => {
                ImageBuffer::<image::Rgb<u16>, Vec<u16>>::from_raw(width, height, p)
                    .and_then(|buf| Some(buf.into()))
            }
            Pixels::Luma8(p) => ImageBuffer::<image::Luma<u8>, Vec<u8>>::from_raw(width, height, p)
                .and_then(|buf| Some(buf.into())),
            Pixels::Luma16(p) => {
                ImageBuffer::<image::Luma<u16>, Vec<u16>>::from_raw(width, height, p)
                    .and_then(|buf| Some(buf.into()))
            }
        } {
            Some(img) => Ok(img),
            None => Err("Unable to convert frame"),
        }
    }
}

impl BinRead for Frame {
    type Args<'a> = FrameFormat;

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let channels: PixelChannels = (&args.color).into();
        let vec_args = VecArgs::builder()
            .count(channels.len() * args.width as usize * args.height as usize)
            .finalize();
        let pixels = match (channels, args.depth) {
            (PixelChannels::Luma, PixelDepth::U8(_)) => {
                Pixels::Luma8(<Vec<u8>>::read_options(reader, endian, vec_args)?)
            }
            (PixelChannels::Luma, PixelDepth::U16(_)) => {
                Pixels::Luma16(<Vec<u16>>::read_options(reader, endian, vec_args)?)
            }
            (PixelChannels::Rgb, PixelDepth::U8(_)) => {
                Pixels::Rgb8(<Vec<u8>>::read_options(reader, endian, vec_args)?)
            }
            (PixelChannels::Rgb, PixelDepth::U16(_)) => {
                Pixels::Rgb16(<Vec<u16>>::read_options(reader, endian, vec_args)?)
            }
        };

        Ok(Frame((args.width, args.height), pixels))
    }
}

impl BinWrite for Frame {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<()> {
        let Frame(_, pixels) = self;
        match pixels {
            Pixels::Rgb8(p) => p.write_options(writer, endian, ()),
            Pixels::Rgb16(p) => p.write_options(writer, endian, ()),
            Pixels::Luma8(p) => p.write_options(writer, endian, ()),
            Pixels::Luma16(p) => p.write_options(writer, endian, ()),
        }
    }
}

impl From<&ColorId> for PixelChannels {
    fn from(value: &ColorId) -> Self {
        match value {
            ColorId::RGB => PixelChannels::Rgb,
            ColorId::BGR => PixelChannels::Rgb,
            _ => PixelChannels::Luma,
        }
    }
}

impl From<&PixelChannels> for ColorId {
    fn from(value: &PixelChannels) -> Self {
        match value {
            PixelChannels::Rgb => ColorId::RGB,
            PixelChannels::Luma => ColorId::MONO,
        }
    }
}

impl From<i32> for PixelDepth {
    fn from(value: i32) -> Self {
        match value {
            v if v > 8 => PixelDepth::U16(v),
            v => PixelDepth::U8(v),
        }
    }
}

impl From<&PixelDepth> for i32 {
    fn from(value: &PixelDepth) -> Self {
        match value {
            PixelDepth::U8(v) => *v,
            PixelDepth::U16(v) => *v,
        }
    }
}
