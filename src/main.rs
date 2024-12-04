use filer::player;

use filer::track;
use std::env;
use std::process;

// Up Next:

// rename Module to TrackerModule
// tm = TrackerModule(filename)
// print(tm.title())
// for s in tm.samples():
//   print(s)

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
                player::play_module(&mut module).await;
            } else if command == "samples" || command == "ss" {
                let period_c3 = 214;
                let period_b3 = 113;
                player::play_samples(&mut module, period_c3);
            }
        }
        Err(e) => eprintln!("Error reading {}: {}", filename, e),
    }
}
