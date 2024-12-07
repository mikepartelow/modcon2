use modcon2::player;

use modcon2::module;
use std::env;
use std::process;

extern crate log;
extern crate pretty_env_logger;

// Up Next:

// rename Module to TrackerModule
// tm = TrackerModule(filename)
// print(tm.title())
// for s in tm.samples():
//   print(s)

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <filename> [command]", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let command = if args.len() == 3 { &args[2] } else { "" };

    match module::read_module(filename) {
        // Ok(module) => player::play_pattern(&module.pattern_table[0]).unwrap(),
        Ok(mut module) => {
            if command.is_empty() {
                let cfg = make_player_config();
                player::play_module(&mut module, cfg).await;
            } else if command == "samples" || command == "ss" {
                let period_c3 = 214;
                let _period_b3 = 113;
                player::play_samples(&mut module, period_c3);
            } else if command == "info" || command == "ii" {
                println!("title: [{}] ({})", module.title, module.title.len());
                println!("---");
                for (i, s) in module.samples.iter().enumerate() {
                    println!("{:02x}: {}", i + 1, s.header);
                }
                println!("---");
                println!("Effects used in this module: "); // FIXME!!!
                println!("FIXME FIXME FIXME!!!");
            }
        }
        Err(e) => eprintln!("Error reading {}: {}", filename, e),
    }
}

fn make_player_config() -> player::Config {
    let key = "PLAY_CHANS";
    let chan_vec = match env::var(key) {
        Ok(val) => {
            let int_vec: Vec<usize> = val
                .split(',')
                .filter_map(|s| s.parse::<usize>().ok())
                .collect();

            int_vec
        }
        Err(_) => vec![0, 1, 2, 3],
    };

    player::Config { channels: chan_vec }
}
