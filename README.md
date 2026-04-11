# ser-file

[![Crates.io](https://img.shields.io/crates/v/ser-file.svg)](https://crates.io/crates/ser-file)
[![Docs.rs](https://docs.rs/ser-file/badge.svg)](https://docs.rs/ser-file)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Read, write, export, create SER files.

This crate includes a lib and a cli (with the `"cli"` feature).

Pre-built binary and library artifacts are available at [repo releases](https://github.com/themadcreator/ser-file/releases).

##### Example: Export SER frames as PNGs

```rust
use ser_file::Ser;
use binrw::BinRead;

use image::DynamicImage;
use std::fs::File;

// Read a SER file
let mut file = File::open("example.ser")?;
let ser = Ser::read(&mut file)?;

// Save each frame as a PNG
for (i, (frame, _timestamp)) in ser.into_iter().enumerate() {
    let img: DynamicImage = frame.try_into()?;
    img.save(format!("frame_{:02}.png", i))?;
}
```

##### Example: Create a SER containing a single PNG

```rust
use ser_file::{Ser, FrameFormat};
use binrw::BinWrite;

use image;
use std::{fs::File, io::BufWriter};

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
```

##### Example: Use the cli

```
> cargo run --features cli --bin ser -- --help
Read, write, export, create SER files

Usage: ser <COMMAND>

Commands:
  info      Prints out information about a SER file
  create    Creates a new SER file from a set of input images
  export    Exports the frames from this SER file
  validate  Validates this library by parsing and writing to memory and comparing the bytes
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```
> cargo run --features cli --bin ser -- info --in example.ser
SER File example.ser
Metadata:
        Observer:   'Observer                                '
        Instrument: 'ZWO ASI385MC                            '
        Telescope:  'Telescope                               '
Datetime:
        Local:  2026-03-08 15:10:11.555915
        UTC:    2026-03-08 22:10:11.555915
Frame Format:
        Color:  BAYER_BGGR
        Depth:  U16(16)
        Width:  1936
        Height: 1096
Frame Count: 10
Frame Timestamps:
        0:  2026-03-08 22:10:11.518220
        1:  2026-03-08 22:10:11.537575
        2:  2026-03-08 22:10:11.557111
        3:  2026-03-08 22:10:11.576502
        4:  2026-03-08 22:10:11.596097
        5:  2026-03-08 22:10:11.615425
        6:  2026-03-08 22:10:11.634807
        7:  2026-03-08 22:10:11.654251
        8:  2026-03-08 22:10:11.673783
        9:  2026-03-08 22:10:11.693369
```




License: MIT
