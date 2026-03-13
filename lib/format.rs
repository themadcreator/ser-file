use binrw::{BinRead, BinResult, BinWrite, VecArgs, binrw};
use getset::{Getters, Setters};
use image::{DynamicImage, GenericImageView, ImageBuffer};

use crate::{ColorId, PixelEndian};

/// Describes the primitive type in which each pixel channel is stored.
#[binrw]
#[br(map(i32::into))]
#[bw(map(i32::from))]
#[derive(Debug, Clone)]
pub enum PixelDepth {
    U8(i32),
    U16(i32),
}

/// Container for different pixel formats
#[derive(Clone)]
pub enum Pixels {
    Rgb8(Vec<u8>),
    Rgb16(Vec<u16>),
    Luma8(Vec<u8>),
    Luma16(Vec<u16>),
}

/// A tuple struct containing the dimensions of an image and its pixels
#[derive(Clone)]
pub struct Frame((u32, u32), Pixels);

/// The format defining all images in a SER file.
///
/// Only one format per SER file is supported. All images must have the same
/// width, height, bytes per pixel, etc.
///
/// Once a format is defined, SER frames may be converted to/from
/// [image::DynamicImage]s. Furthermore, a [FrameFormat] may be computed from an
/// existing [DynamicImage] using [TryFrom].
#[derive(Clone, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct FrameFormat {
    color: ColorId,
    depth: PixelDepth,
    endian: PixelEndian,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone)]
enum PixelChannels {
    Rgb,
    Luma,
}

impl FrameFormat {
    pub fn new(
        color: ColorId,
        depth: PixelDepth,
        endian: PixelEndian,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            color,
            depth,
            endian,
            width,
            height,
        }
    }

    pub fn raw_len(&self) -> usize {
        let channels: PixelChannels = (&self.color).into();
        channels.len() * self.width as usize * self.height as usize
    }

    /// Attempt to convert a [DynamicImage] into a [Frame] that is compatible
    /// with this [FrameFormat].
    pub fn try_into_frame(&self, img: DynamicImage) -> Result<Frame, &'static str> {
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
}

impl PixelChannels {
    fn len(&self) -> usize {
        match self {
            PixelChannels::Rgb => 3,
            PixelChannels::Luma => 1,
        }
    }
}

impl Pixels {
    fn len(&self) -> usize {
        match self {
            Pixels::Rgb8(items) => items.len(),
            Pixels::Rgb16(items) => items.len(),
            Pixels::Luma8(items) => items.len(),
            Pixels::Luma16(items) => items.len(),
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
                PixelEndian::host_endian(),
                width,
                height,
            )),
            DynamicImage::ImageRgb16(_) => Ok(FrameFormat::new(
                ColorId::RGB,
                PixelDepth::U16(16),
                PixelEndian::host_endian(),
                width,
                height,
            )),
            DynamicImage::ImageLuma8(_) => Ok(FrameFormat::new(
                ColorId::MONO,
                PixelDepth::U8(8),
                PixelEndian::host_endian(),
                width,
                height,
            )),
            DynamicImage::ImageLuma16(_) => Ok(FrameFormat::new(
                ColorId::MONO,
                PixelDepth::U16(16),
                PixelEndian::host_endian(),
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

impl PartialEq<Frame> for FrameFormat {
    fn eq(&self, frame: &Frame) -> bool {
        let Frame((w, h), p) = frame;
        &self.width == w && &self.height == h && self.raw_len() == p.len() && p == self.depth
    }
}

impl PartialEq<PixelDepth> for &Pixels {
    fn eq(&self, depth: &PixelDepth) -> bool {
        match self {
            Pixels::Rgb8(_) => matches!(depth, PixelDepth::U8(_)),
            Pixels::Rgb16(_) => matches!(depth, PixelDepth::U16(_)),
            Pixels::Luma8(_) => matches!(depth, PixelDepth::U8(_)),
            Pixels::Luma16(_) => matches!(depth, PixelDepth::U16(_)),
        }
    }
}
