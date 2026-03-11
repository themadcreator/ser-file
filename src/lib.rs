// SPDX-License-Identifier: MIT

#![doc = include_str!("../README.md")]

//! 
//! ## Example: Export SER frames as PNGs
//! 
//! ```rust,no_run
//! use ser_file::Ser;
//! use binrw::BinRead;
//! 
//! use image::DynamicImage;
//! use std::fs::File;
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! 
//! // Read a SER file
//! let mut file = File::open("example.ser")?;
//! let ser = Ser::read(&mut file)?;
//! 
//! // Save each frame as a PNG
//! for (i, (frame, _timestamp)) in ser.iter().enumerate() {
//!     let img: DynamicImage = frame.clone().try_into()?;
//!     img.save(format!("frame_{:02}.png", i))?;
//! }
//! 
//! # Ok(())
//! # }
//! ```
//! 
//! ## Example: Create a SER containing a single PNG
//! 
//! ```rust,no_run
//! use binrw::BinWrite;
//! use ser_file::{Ser, FrameFormat};
//! 
//! use image;
//! use std::{fs::File, io::BufWriter};
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! 
//! // Read a PNG
//! let img = image::open("example.png")?;
//! 
//! // Create a format matching the image
//! let format: FrameFormat = (&img).try_into()?;
//! 
//! // Create a new SER document
//! let mut ser = Ser::with_format(format);
//! 
//! // Add image as a frame
//! let mut frames = ser.frames_mut();
//! let frame = frames.format().try_into_frame(img)?;
//! frames.try_push(frame, None)?;
//! 
//! // Write
//! let mut out = BufWriter::new(File::create("output.ser")?);
//! ser.write(&mut out)?;
//! 
//! # Ok(())
//! # }
//! ```
//! 

mod fixed_string;
mod format;
mod ser;
mod timestamp;
pub use fixed_string::*;
pub use format::*;
pub use ser::*;
pub use timestamp::*;
