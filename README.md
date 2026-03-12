[![Crates.io](https://img.shields.io/crates/v/ser-file.svg)](https://crates.io/crates/ser-file)
[![Docs.rs](https://docs.rs/ser-file/badge.svg)](https://docs.rs/ser-file)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Read, write, export, create SER files.

This crate includes a lib and a cli.

To use the cli, build this crate with `--examples` or download a pre-built
artifact from the repo.

#### Example: Export SER frames as PNGs

```rust,no_run
use ser_file::Ser;
use binrw::BinRead;

use image::DynamicImage;
use std::fs::File;
# fn example() -> Result<(), Box<dyn std::error::Error>> {

// Read a SER file
let mut file = File::open("example.ser")?;
let ser = Ser::read(&mut file)?;

// Save each frame as a PNG
for (i, (frame, _timestamp)) in ser.into_iter().enumerate() {
    let img: DynamicImage = frame.try_into()?;
    img.save(format!("frame_{:02}.png", i))?;
}

# Ok(())
# }
```

#### Example: Create a SER containing a single PNG

```rust,no_run
use binrw::BinWrite;
use ser_file::{Ser, FrameFormat};

use image;
use std::{fs::File, io::BufWriter};
# fn example() -> Result<(), Box<dyn std::error::Error>> {

// Read a PNG
let img = image::open("example.png")?;

// Create a format matching the image
let format = FrameFormat::try_from(&img)?;

// Create a new SER document
let mut ser = Ser::with_format(format);

// Add image as a frame
let mut frames = ser.frames_mut();
let frame = frames.format().try_into_frame(img)?;
frames.try_push(frame, None)?;

// Write
let mut out = BufWriter::new(File::create("output.ser")?);
ser.write(&mut out)?;

# Ok(())
# }
```