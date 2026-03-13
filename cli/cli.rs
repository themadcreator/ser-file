use binrw::{BinRead, BinWrite};
use clap::{Parser, Subcommand};
use glob::glob;
use image::DynamicImage;
use std::{
    fs::{File, FileTimes},
    io::{BufWriter, Cursor, Read},
    path::PathBuf,
};

use ser_file::{FrameFormat, Ser};

/// Read and write SER files
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Info(InfoCommand),
    Create(CreateCommand),
    Export(ExportCommand),
    Validate(ValidateCommand),
}

/// Prints out information about a SER file
#[derive(Parser, Clone)]
struct InfoCommand {
    /// Input SER file
    #[arg(long, short, aliases = ["in"])]
    input: String,
}

/// Creates a new SER file from a set of input images.
///
/// The images may be any type readable by the `image` crate, but they must have the same width and height.
///
/// SER supports up to [u8] and [u16] samples. Any other data will be down converted.
#[derive(Parser, Clone)]
struct CreateCommand {
    /// Input image files. These may contain glob patterns and there may be multiple '--in' args.
    #[arg(long, short, aliases = ["in"])]
    input: Vec<String>,

    /// Output SER file
    #[arg(long, short, default_value = "out.ser")]
    out: String,
}

/// Exports the frames from this SER file.
#[derive(Parser, Clone)]
struct ExportCommand {
    /// Input SER file
    #[arg(long, short, aliases = ["in"])]
    input: String,

    /// Output file extention from the `image` crate. Default: "png"
    ///
    /// Output frames will be named `[ser_file_basename]_000.[ext]`, `[ser_file_basename]_001.[ext]`, ...
    ///
    /// See: <https://crates.io/crates/image>
    #[arg(long, short, default_value = "png")]
    ext: String,

    /// Optionally sets the create timestamp of the images to the frame timestamp (if available)
    #[arg(long, short, default_value_t = false)]
    timestamp: bool,
}

/// Validates this library by parsing and writing to memory and comparing the bytes.
#[derive(Parser, Clone)]
struct ValidateCommand {
    /// Input SER file
    #[arg(long, short, aliases = ["in"])]
    input: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Info(cmd) => {
            let mut file = File::open(&cmd.input)?;
            let ser = Ser::read(&mut file)?;
            println!("SER File {}", cmd.input);

            println!("Metadata:");
            println!("\tObserver:   '{}'", ser.observer());
            println!("\tInstrument: '{}'", ser.instrument());
            println!("\tTelescope:  '{}'", ser.telescope());

            println!("Datetime:");
            println!("\tLocal:  {}", ser.datetime());
            println!("\tUTC:    {}", ser.datetime_utc());

            let format = ser.frame_format();
            println!("Frame Format:");
            println!("\tColor:  {:?}", format.color());
            println!("\tDepth:  {:?}", format.depth());
            println!("\tWidth:  {}", format.width());
            println!("\tHeight: {}", format.height());

            println!("Frame Count: {}", ser.frame_count());

            if ser.has_frame_timestamps() {
                println!("Frame Timestamps:");
                for (i, (_f, t)) in ser.iter().enumerate() {
                    println!("\t{}:  {}", i, t.unwrap());
                }
            }
        }
        Commands::Create(cmd) => {
            let mut files: Vec<PathBuf> = Vec::new();
            for input in cmd.input {
                for entry in glob(input.as_str())? {
                    files.push(entry?);
                }
            }

            let mut images: Vec<DynamicImage> = Vec::new();
            for file in files {
                images.push(image::open(file)?);
            }

            if images.len() == 0 {
                return Err("At least one image must be provided, but none were found.".into());
            }

            let first = images.iter().next().unwrap();
            let format: FrameFormat = first.try_into()?;

            let mut ser = Ser::with_format(format);
            let mut frames = ser.frames_mut();
            for img in images {
                frames.try_push(frames.format().try_into_frame(img)?, None)?;
            }

            let mut out_file = BufWriter::new(File::create(cmd.out)?);
            ser.write(&mut out_file)?;
        }
        Commands::Export(cmd) => {
            let input_path: PathBuf = cmd.input.into();
            let stem = input_path
                .file_stem()
                .ok_or("Could not parse filename")?
                .to_str()
                .ok_or("Could not convert OsStr")?;

            let mut file = File::open(&input_path)?;
            let ser = Ser::read(&mut file)?;

            for (i, (frame, timestamp)) in ser.iter().enumerate() {
                let img: DynamicImage = frame.clone().try_into()?;

                let filename: PathBuf = format!("{}_{:03}.{}", stem, i, cmd.ext).into();
                println!("Writing {:?}", &filename);
                img.save(&filename)?;

                if let Some(ts) = timestamp
                    && cmd.timestamp
                {
                    let file = File::open(&filename)?;
                    let times = FileTimes::new().set_modified(ts.try_into()?);
                    file.set_times(times)?;
                }
            }
        }
        Commands::Validate(cmd) => {
            let mut file = File::open(cmd.input)?;
            let mut in_buf: Vec<u8> = Vec::new();
            file.read_to_end(&mut in_buf)?;

            let ser = Ser::read(&mut Cursor::new(&mut in_buf))?;
            let mut out_buf: Vec<u8> = Vec::new();
            ser.write(&mut Cursor::new(&mut out_buf))?;

            if in_buf != out_buf {
                return Err("SER read/write failed to round trip. Bytes differ.".into());
            }

            println!("SER round trip successful. Bytes match exactly.");
        }
    }

    Ok(())
}
