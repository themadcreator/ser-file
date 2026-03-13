use binrw::binrw;
use getset::{Getters, Setters};
use std::fmt::{Display, Formatter};

use crate::{FixedString, Frame, FrameFormat, PixelDepth, Timestamp};

/// SER format description version 3
///
/// Authors
/// Heiko Wilkens (version 2)
/// Grischa Hahn (red = extensions of version 3)
///
/// 2014 Feb 06
///
/// Source: <https://grischa-hahn.hier-im-netz.de/astro/ser/SER%20Doc%20V3b.pdf>
///
#[binrw]
#[brw(little)]
#[derive(Getters, Setters)]
pub struct Ser {
    /// 1_FileID
    ///
    /// Format: String
    ///
    /// Length: 14 Byte (14 ASCII characters)
    ///
    /// Content: "LUCAM-RECORDER" (fix)
    #[brw(magic = b"LUCAM-RECORDER")]
    file_id: (),

    /// 2_LuID
    ///
    /// Format: Integer_32 (little-endian)
    ///
    /// Length: 4 Byte
    ///
    /// Content: Lumenera camera series ID (currently unused; default = 0)
    #[getset(get = "pub", set = "pub")]
    lu_id: i32,

    /// 3_ColorID
    ///
    /// Format: Integer_32 (little-endian)
    ///
    /// Length: 4 Byte
    color_id: ColorId,

    /// 4_LittleEndian
    ///
    /// Format: Integer_32 (little-endian)
    ///
    /// Length: 4 Byte
    ///
    /// Content: 0 (FALSE) for big-endian byte order in 16 bit image data
    /// 1 (TRUE) for little-endian byte order in 16 bit image data
    little_endian: PixelEndian,

    /// 5_ImageWidth
    ///
    /// Format: Integer_32 (little-endian)
    ///
    /// Length: 4 Byte
    ///
    /// Content: Width of every image in pixel
    image_width: i32,

    /// 6_ImageHeight
    ///
    /// Format: Integer_32 (little-endian)
    ///
    /// Length: 4 Byte
    ///
    /// Content: Height of every image in pixel
    image_height: i32,

    /// 7_PixelDepthPerPlane
    ///
    /// Format: Integer_32 (little-endian)
    ///
    /// Length: 4 Byte
    ///
    /// Content: True bit depth per pixel per plane
    pixel_depth_per_plane: PixelDepth,

    /// 8_FrameCount
    ///
    /// Format: Integer_32 (little-endian)
    ///
    /// Length: 4 Byte
    ///
    /// Content: Number of image frames in SER file
    #[br(temp)]
    #[bw(calc(image_data.len() as _))]
    frame_count: i32,

    /// 9_Observer
    ///
    /// Format: String
    ///
    /// Length: 40 Byte (40 ASCII characters {32…126 dec.}, fill unused characters with 0 dec.)
    ///
    /// Content: Name of observer
    #[getset(get = "pub", set = "pub")]
    observer: FixedString<40>,

    /// 10_Instrument
    ///
    /// Format: String
    ///
    /// Length: 40 Byte (40 ASCII characters {32…126 dec.}, fill unused characters with 0 dec.)
    ///
    /// Content: Name of used camera
    #[getset(get = "pub", set = "pub")]
    instrument: FixedString<40>,

    /// 11_Telescope
    ///
    /// Format: String
    ///
    /// Length: 40 Byte (40 ASCII characters {32…126 dec.}, fill unused characters with 0 dec.)
    ///
    /// Content: Name of used telescope
    #[getset(get = "pub", set = "pub")]
    telescope: FixedString<40>,

    /// 12_DateTime
    ///
    /// Format: Date / Integer_64 (little-endian)
    ///
    /// Length: 8 Byte
    ///
    /// Content: Start time of image stream (local time)
    ///
    /// If 12_DateTime <= 0 then 12_DateTime is invalid and the SER file does not contain a
    /// Time stamp trailer.
    #[getset(get = "pub")]
    datetime: Timestamp,

    /// 13_DateTime_UTC
    ///
    /// Format: Date / Integer_64 (little-endian)
    ///
    /// Length: 8 Byte
    ///
    /// Content: Start time of image stream in UTC
    #[getset(get = "pub")]
    datetime_utc: Timestamp,

    /// Image Data
    ///
    /// Image data starts at File start offset decimal 178
    ///
    /// Size of every image frame in byte is: 5_ImageWidth x 6_ImageHeigth x BytePerPixel
    #[br(args {
        count: frame_count as usize,
        inner: FrameFormat::new(
            color_id.clone(),
            pixel_depth_per_plane.clone(),
            little_endian.clone(),
            image_width as _,
            image_height as _,
        )
    })]
    image_data: Vec<Frame>,

    /// Trailer
    ///
    /// Trailer starts at byte offset: 178 + 8_FrameCount x 5_ImageWidth x 6_ImageHeigth x
    /// BytePerPixel.
    ///
    /// Trailer contains Date / Integer_64 (little-endian) time stamps in UTC for every image frame.
    /// According to Microsoft documentation the used time stamp has the following format:
    /// “Holds IEEE 64-bit (8-byte) values that represent dates ranging from January 1 of the year 0001
    /// through December 31 of the year 9999, and times from 12:00:00 AM (midnight) through
    /// 11:59:59.9999999 PM. Each increment represents 100 nanoseconds of elapsed time since the
    /// beginning of January 1 of the year 1 in the Gregorian calendar. The maximum value represents
    /// 100 nanoseconds before the beginning of January 1 of the year 10000.”
    ///
    /// According to the findings of Raoul Behrend, Université de Genève, the date record is not a 64 bits
    /// unsigned integer as stated, but a 62 bits unsigned integer. He got no information about the use of
    /// the two MSB.
    #[br(count = match frame_count { f if datetime.is_valid() => f as usize, _ => 0})]
    trailer: Vec<Timestamp>,
}

/// Mutate SER file datetimes and frame timestamps
pub struct DatesMut<'a> {
    datetime: &'a mut Timestamp,
    datetime_utc: &'a mut Timestamp,
    frame_count: usize,
    frame_times: &'a mut Vec<Timestamp>,
}

#[derive(Debug)]
pub enum DateErrors {
    InvalidDatetime,
    IncorrectTimestamps,
}

/// Mutate SER file frames
pub struct FramesMut<'a> {
    format: FrameFormat,
    has_trailer: bool,
    frames: &'a mut Vec<Frame>,
    frame_times: &'a mut Vec<Timestamp>,
}

#[derive(Debug)]
pub enum FramePushErrors {
    Incompatible,
    TimestampExpected,
    TimestampUnexpected,
}

/// Describes the color format of SER frames
#[binrw]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum ColorId {
    #[brw(magic = 0i32)]
    MONO,
    #[brw(magic = 8i32)]
    BAYER_RGGB,
    #[brw(magic = 9i32)]
    BAYER_GRBG,
    #[brw(magic = 10i32)]
    BAYER_GBRG,
    #[brw(magic = 11i32)]
    BAYER_BGGR,
    #[brw(magic = 16i32)]
    BAYER_CYYM,
    #[brw(magic = 17i32)]
    BAYER_YCMY,
    #[brw(magic = 18i32)]
    BAYER_YMCY,
    #[brw(magic = 19i32)]
    BAYER_MYYC,
    #[brw(magic = 100i32)]
    RGB,
    #[brw(magic = 101i32)]
    BGR,
}

/// Describes the endianness of pixel data in SER frames
#[binrw]
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub enum PixelEndian {
    #[brw(magic = 0i32)]
    Big,
    #[brw(magic = 1i32)]
    Little,
}

impl Ser {
    pub fn with_format(format: FrameFormat) -> Self {
        Self {
            file_id: (),
            lu_id: 0,
            color_id: format.color().clone(),
            little_endian: format.endian().clone(),
            image_width: *format.width() as _,
            image_height: *format.height() as _,
            pixel_depth_per_plane: format.depth().clone(),
            observer: FixedString::default(),
            instrument: FixedString::default(),
            telescope: FixedString::default(),
            datetime: Timestamp::default(),
            datetime_utc: Timestamp::default(),
            image_data: Vec::new(),
            trailer: Vec::new(),
        }
    }

    /// Creates a new copy of this SER's frame format
    pub fn frame_format(&self) -> FrameFormat {
        FrameFormat::new(
            self.color_id.clone(),
            self.pixel_depth_per_plane.clone(),
            self.little_endian.clone(),
            self.image_width as _,
            self.image_height as _,
        )
    }

    /// SER files include timestamps for each frame only if the `datetime` field
    /// is valid.
    pub fn has_frame_timestamps(&self) -> bool {
        self.datetime.is_valid()
    }

    /// The number of frames
    pub fn frame_count(&self) -> usize {
        self.image_data.len()
    }

    /// Returns an iterator for [Frame] references and their associated
    /// [Timestamp] if provided.
    pub fn iter(&self) -> impl Iterator<Item = (&Frame, Option<&Timestamp>)> {
        let mut times = self.trailer.iter();
        self.image_data
            .iter()
            .zip(std::iter::from_fn(move || Some(times.next())))
    }

    /// Moves into an iterator for [Frame]s and their associated [Timestamp] if
    /// provided.
    pub fn into_iter(self) -> impl Iterator<Item = (Frame, Option<Timestamp>)> {
        let mut times = self.trailer.into_iter();
        self.image_data
            .into_iter()
            .zip(std::iter::from_fn(move || Some(times.next())))
    }

    /// Returns a [FramesMut] object for mutating frames
    pub fn frames_mut<'a>(&'a mut self) -> FramesMut<'a> {
        FramesMut {
            format: self.frame_format(),
            has_trailer: self.has_frame_timestamps(),
            frames: &mut self.image_data,
            frame_times: &mut self.trailer,
        }
    }

    /// Returns a [DatesMut] object for mutating dates and frame timestamps
    pub fn dates_mut<'a>(&'a mut self) -> DatesMut<'a> {
        DatesMut {
            datetime: &mut self.datetime,
            datetime_utc: &mut self.datetime_utc,
            frame_count: self.image_data.len(),
            frame_times: &mut self.trailer,
        }
    }
}

impl<'a> FramesMut<'a> {
    /// Returns a reference to the current [FrameFormat].
    pub fn format(&self) -> &FrameFormat {
        &self.format
    }

    /// Push a frame onto the frames [Vec].
    ///
    /// [Frame]s must have a pixel format compatible with self's [FrameFormat].
    ///
    /// Frame [Timestamp]s MUST be specified if the SER's datetime is set.
    /// Otherwise, they MAY NOT be specified.
    ///
    /// Frame timestamps are in UTC
    pub fn try_push(
        &mut self,
        frame: Frame,
        timestamp: Option<Timestamp>,
    ) -> Result<(), FramePushErrors> {
        if self.format != frame {
            return Err(FramePushErrors::Incompatible);
        }

        if self.has_trailer {
            match timestamp {
                Some(ts) => self.frame_times.push(ts),
                None => return Err(FramePushErrors::TimestampExpected),
            }
        } else {
            match timestamp {
                Some(_) => return Err(FramePushErrors::TimestampUnexpected),
                None => (),
            }
        };

        self.frames.push(frame);

        Ok(())
    }
}

impl<'a> DatesMut<'a> {
    /// Clear the file's datetimes and frame timestamps
    pub fn clear(&mut self) {
        *self.datetime = Timestamp::default();
        *self.datetime_utc = Timestamp::default();
        self.frame_times.clear();
    }

    /// Sets the file's datetimes and frame timestamps.
    ///
    /// The provided datetimes MUST be valid [Timestamp]s.
    ///
    /// The [Vec] of timestamps MUST have the same length as frames in the file.
    pub fn try_set_dates(
        &mut self,
        datetime: Timestamp,
        datetime_utc: Timestamp,
        frame_times: Vec<Timestamp>,
    ) -> Result<(), DateErrors> {
        if !datetime.is_valid() || !datetime_utc.is_valid() {
            return Err(DateErrors::InvalidDatetime);
        }

        if frame_times.len() != self.frame_count {
            return Err(DateErrors::IncorrectTimestamps);
        }

        *self.datetime = datetime;
        *self.datetime_utc = datetime_utc;
        *self.frame_times = frame_times;

        Ok(())
    }
}

impl std::error::Error for FramePushErrors {}

impl Display for FramePushErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FramePushErrors::Incompatible => {
                f.write_str("Frame incompatible. All frames must have the same format.")
            }
            FramePushErrors::TimestampExpected => f.write_str(
                "Timestamps MUST be added for each frame when the SER's datetime is valid.",
            ),
            FramePushErrors::TimestampUnexpected => {
                f.write_str("Timestamps MAY NOT be added when the SER's datetime is invalid.")
            }
        }
    }
}

impl std::error::Error for DateErrors {}

impl Display for DateErrors {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DateErrors::InvalidDatetime => f.write_str("Cannot set datetime to invalid timestamp. To clear the datetimes, use the `.clear()` method."),
            DateErrors::IncorrectTimestamps => f.write_str("Frame timestamps do not match the number of the frames."),
        }
    }
}

impl PixelEndian {
    pub fn host_endian() -> Self {
        if cfg!(target_endian = "big") {
            PixelEndian::Big
        } else {
            PixelEndian::Little
        }
    }
}

impl From<FrameFormat> for Ser {
    fn from(value: FrameFormat) -> Self {
        Self::with_format(value)
    }
}
