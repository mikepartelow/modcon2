use modcon2::player;

use modcon2::module;
use std::env;
use std::fs::File;
use std::process;

extern crate log;
extern crate pretty_env_logger;

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

    let file = File::open(filename).unwrap();

    match module::read(file) {
        Ok(mut module) => {
            if command.is_empty() {
                let cfg = make_player_config();
                player::play_module(&mut module, cfg).await;
            } else if command == "samples" || command == "ss" {
                let period_c3 = 214;
                player::play_samples(&mut module, period_c3);
            } else if command == "info" || command == "ii" {
                println!("title: [{}] ({})", module.title, module.title.len());
                println!("---");
                for (i, s) in module.samples.iter().enumerate() {
                    println!("{:02x}: {}", i + 1, s);
                }
            } else if command == "todo" {
                println!("Unit tests");
                println!("Integration tests");
                println!("FIXMEs");
                println!("sample volumes");
                println!("remove all unwraps() and other things that panic");
                println!("Effects used in this module but not yet implemented in modcon2: ");
                // FIXME ^^^^^^
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
