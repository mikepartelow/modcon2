use filer::player;
use filer::track;
use std::env;
use std::process;

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

use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <filename> [command]", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let command = if args.len() == 3 { &args[2] } else { "" };

    match track::read_module(filename) {
        // Ok(module) => player::play_pattern(&module.pattern_table[0]).unwrap(),
        Ok(mut module) => {
            if command == "" {
                player::play_module(&mut module, 2).await;
            } else if command == "samples" || command == "ss" {
                let period_c3 = 214;
                let period_b3 = 113;
                player::play_samples(&mut module, period_c3);
            } else if command == "notes" || command == "nn" {
                player::play_module_notes(&mut module, 2).await;
            }
        }
        Err(e) => eprintln!("Error reading {}: {}", filename, e),
    }
}
