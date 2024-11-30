use filer::track;
use rodio::Source;
use std::env;
use std::process;
use std::str::FromStr;

use rodio::{source::SineWave, OutputStream, Sink};

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

// fn main() -> io::Result<()> {
//     let args: Vec<String> = env::args().collect();
//     if args.len() != 2 {
//         eprintln!("Usage: {} <filename>", args[0]);
//         process::exit(1);
//     }

//     let filename = &args[1];
//     match track::read_module(filename) {
//         // Ok(module) => player::play_pattern(&module.pattern_table[0]).unwrap(),
//         Ok(mut module) => player::play(&mut module).unwrap(),
//         Err(e) => eprintln!("Error reading {}: {}", filename, e),
//     }

//     // noise();

//     Ok(())
// }

use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    match track::read_module(filename) {
        // Ok(module) => player::play_pattern(&module.pattern_table[0]).unwrap(),
        Ok(mut module) => {
            let mut interval = time::interval(Duration::from_millis(20 * 6));

            for (i, &pidx) in module.pattern_table.iter().enumerate() {
                if i == 73 {
                    break; // FIXME: knulla-specific hack, replace with module.num_patterns
                }

                // FIXME: iterate to m.num_patterns - the actual number of patterns, not 128
                let print_prefix = format!(
                    "{:03}/{:03} P{:03} ",
                    i,
                    module.pattern_table.len() - 1,
                    pidx
                ); // FIXME: rustier than this

                let p: &mut track::Pattern = &mut module.patterns[pidx as usize];
                while let Some((row, channels)) = p.next() {
                    let mut row_str = String::from_str(&format!("R{:02}:", row))
                        .expect("FIXME: expect is discouraged");

                    for ch in channels {
                        row_str += &format!(
                            "|{} {:02x} {:04x} {:04x}",
                            ch.note, ch.sample, ch.period, ch.effect
                        );
                        row_str += &"|";
                    }

                    println!("{} {}", print_prefix, row_str);
                    interval.tick().await; // FIXME: would a sleep be simpler? is any delay even necessary? does playing N ticks of queued audio provide the necessary delay?
                }
            }
        }
        Err(e) => eprintln!("Error reading {}: {}", filename, e),
    }
}
// // https://github.com/Prezzodaman/pymod/blob/main/pymod/pymod.py
// // https://www.ocf.berkeley.edu/~eek/index.html/tiny_examples/ptmod/
// // https://github.com/cmatsuoka/tracker-history

// //         // https://modarchive.org/forums/index.php?topic=2709.0
