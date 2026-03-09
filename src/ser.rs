use binrw::binrw;

use crate::{FixedString, Frame, FrameFormat, PixelDepth, Timestamp};

/// SER format description version 3
/// Authors
/// Heiko Wilkens (version 2)
/// Grischa Hahn (red = extensions of version 3)
/// 2014 Feb 06
///
/// Source: <https://grischa-hahn.hier-im-netz.de/astro/ser/SER%20Doc%20V3b.pdf>
///
#[binrw]
#[brw(little)]
pub struct Ser {
    /// 1_FileID
    ///
    /// Format: String
    /// Length: 14 Byte (14 ASCII characters)
    /// Content: "LUCAM-RECORDER" (fix)
    #[brw(magic = b"LUCAM-RECORDER")]
    file_id: (),

    /// 2_LuID
    ///
    /// Format: Integer_32 (little-endian)
    /// Length: 4 Byte
    /// Content: Lumenera camera series ID (currently unused; default = 0)
    lu_id: i32,

    /// 3_ColorID
    ///
    /// Format: Integer_32 (little-endian)
    /// Length: 4 Byte
    color_id: ColorId,

    /// 4_LittleEndian
    ///
    /// Format: Integer_32 (little-endian)
    /// Length: 4 Byte
    /// Content: 0 (FALSE) for big-endian byte order in 16 bit image data
    /// 1 (TRUE) for little-endian byte order in 16 bit image data
    little_endian: PixelEndianness,

    /// 5_ImageWidth
    ///
    /// Format: Integer_32 (little-endian)
    /// Length: 4 Byte
    /// Content: Width of every image in pixel
    image_width: i32,

    /// 6_ImageHeight
    ///
    /// Format: Integer_32 (little-endian)
    /// Length: 4 Byte
    /// Content: Height of every image in pixel
    image_height: i32,

    /// 7_PixelDepthPerPlane
    ///
    /// Format: Integer_32 (little-endian)
    /// Length: 4 Byte
    /// Content: True bit depth per pixel per plane
    pixel_depth_per_plane: PixelDepth,

    /// 8_FrameCount
    ///
    /// Format: Integer_32 (little-endian)
    /// Length: 4 Byte
    /// Content: Number of image frames in SER file
    frame_count: i32,

    /// 9_Observer
    ///
    /// Format: String
    /// Length: 40 Byte (40 ASCII characters {32…126 dec.}, fill unused characters with 0 dec.)
    /// Content: Name of observer
    observer: FixedString<40>,

    /// 10_Instrument
    ///
    /// Format: String
    /// Length: 40 Byte (40 ASCII characters {32…126 dec.}, fill unused characters with 0 dec.)
    /// Content: Name of used camera
    instrument: FixedString<40>,

    /// 11_Telescope
    ///
    /// Format: String
    /// Length: 40 Byte (40 ASCII characters {32…126 dec.}, fill unused characters with 0 dec.)
    /// Content: Name of used telescope
    telescope: FixedString<40>,

    /// 12_DateTime
    ///
    /// Format: Date / Integer_64 (little-endian)
    /// Length: 8 Byte
    /// Content: Start time of image stream (local time)
    /// If 12_DateTime <= 0 then 12_DateTime is invalid and the SER file does not contain a
    /// Time stamp trailer.
    datetime: Timestamp,

    /// 13_DateTime_UTC
    ///
    /// Format: Date / Integer_64 (little-endian)
    /// Length: 8 Byte
    /// Content: Start time of image stream in UTC
    datetime_utc: Timestamp,

    /// Image Data
    ///
    /// Image data starts at File start offset decimal 178
    /// Size of every image frame in byte is: 5_ImageWidth x 6_ImageHeigth x BytePerPixel
    #[br(args {
        count: frame_count as usize,
        inner: FrameFormat {
            color: color_id.clone(),
            depth: pixel_depth_per_plane.clone(),
            width: image_width as _,
            height: image_height as _,
        }
    })]
    image_data: Vec<Frame>,

    /// Trailer
    ///
    /// Trailer starts at byte offset: 178 + 8_FrameCount x 5_ImageWidth x 6_ImageHeigth x
    /// BytePerPixel.
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

impl Ser {
    pub fn new(format: FrameFormat) -> Self {
        Self {
            file_id: (),
            lu_id: 0,
            color_id: format.color.clone(),
            little_endian: PixelEndianness::Little,
            image_width: format.width as _,
            image_height: format.height as _,
            pixel_depth_per_plane: format.depth.into(),
            frame_count: 0,
            observer: FixedString([0u8; 40]),
            instrument: FixedString([0u8; 40]),
            telescope: FixedString([0u8; 40]),
            datetime: Timestamp::default(),
            datetime_utc: Timestamp::default(),
            image_data: Vec::new(),
            trailer: Vec::new(),
        }
    }

    pub fn frame_format(&self) -> FrameFormat {
        FrameFormat {
            color: self.color_id.clone(),
            depth: self.pixel_depth_per_plane.clone(),
            width: self.image_width as _,
            height: self.image_height as _,
        }
    }

    pub fn has_timestamp_trailer(&self) -> bool {
        self.datetime.is_valid()
    }

    pub fn iter_frames(&self) -> std::slice::Iter<'_, Frame> {
        self.image_data.iter()
    }

    pub fn iter_trailer(&self) -> std::slice::Iter<'_, Timestamp> {
        self.trailer.iter()
    }

    pub fn add_frame(&mut self, frame: Frame, timestamp: Option<Timestamp>) {
        self.image_data.push(frame);
        self.frame_count += 1;
        if self.has_timestamp_trailer() {
            self.trailer.push(timestamp.unwrap_or_default())
        }
    }

    pub fn frame_count(&self) -> &i32 {
        &self.frame_count
    }

    pub fn color_id(&self) -> &ColorId {
        &self.color_id
    }

    pub fn pixel_depth_per_plane(&self) -> &PixelDepth {
        &self.pixel_depth_per_plane
    }

    pub fn observer(&self) -> &FixedString<40> {
        &self.observer
    }

    pub fn telescope(&self) -> &FixedString<40> {
        &self.telescope
    }

    pub fn instrument(&self) -> &FixedString<40> {
        &self.instrument
    }

    pub fn datetime(&self) -> &Timestamp {
        &self.datetime
    }

    pub fn datetime_utc(&self) -> &Timestamp {
        &self.datetime_utc
    }
}

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

#[binrw]
#[allow(non_camel_case_types)]
pub enum PixelEndianness {
    #[brw(magic = 0i32)]
    Big,
    #[brw(magic = 1i32)]
    Little,
}

impl From<FrameFormat> for Ser {
    fn from(value: FrameFormat) -> Self {
        Self::new(value)
    }
}
