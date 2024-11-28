use filer::player;
use filer::track;
use rodio::Source;
use std::env;
use std::io;
use std::process;

use rodio::{source::SineWave, OutputStream, Sink};
use std::time::Duration;

// https://www.aes.id.au/modformat.html
// https://modarchive.org/index.php?request=view_by_moduleid&query=48107
// https://web.archive.org/web/20100921225940/http://io.debian.net/~tar/debian/xmp/xmp-2.7.1/docs/formats/Ultimate_Soundtracker-format.txt
// https://github.com/mikepartelow/rust-chess/tree/main/app/src

// https://github.com/cmatsuoka/oxdz
// https://github.com/libxmp/libxmp

// Up Next:

// rename Module to TrackerModule
// tm = TrackerModule(filename)
// print(tm.title())
// for s in tm.samples():
//   print(s)

fn noise() {
    // Create a new output stream and stream handle
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // Create a new sink
    let sink = Sink::try_new(&stream_handle).unwrap();

    // // Generate a sine wave of 440 Hz (A4 note)
    // let source = SineWave::new(440);

    // Play the sine wave for 2 seconds
    sink.append(SineWave::new(440).take_duration(Duration::from_secs(1)));
    sink.append(SineWave::new(420).take_duration(Duration::from_secs(1)));
    sink.append(SineWave::new(440).take_duration(Duration::from_secs(1)));

    // Sleep the thread to let the sound play
    std::thread::sleep(Duration::from_secs(3));
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    match track::read_module(filename) {
        Ok(module) => player::play_samples(module).unwrap(),
        Err(e) => eprintln!("Error reading {}: {}", filename, e),
    }

    // noise();

    Ok(())
}
