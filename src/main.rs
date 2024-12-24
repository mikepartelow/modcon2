use modcon2::player;

use modcon2::module;
use std::env;
use std::fs::File;
use std::process;
use std::time::Duration;

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
                let effects = player::play_module(&mut module, cfg).await;

                let mut effects = effects.into_iter().collect::<Vec<_>>();
                effects.sort();

                for effect in effects {
                    println!(" effect: {:04x}", effect);
                }
            } else if command == "samples" || command == "ss" {
                let period_c3 = 214;
                player::play_samples(&mut module, period_c3);
            } else if command == "as" {
                let sample_num = env::var("SAMPLE_NUM")
                    .unwrap()
                    .parse::<usize>()
                    .unwrap()
                    .saturating_sub(1);
                let sample = &module.samples[sample_num];

                let period_c3 = 214;
                // let semi_x = 4;
                // let semi_y = 7;

                println!("oh: {}", sample_num);

                player::play_sample(sample, period_c3, false);
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

    // FIXME: this must all go away and be replaced by reading the tempo effect from the file
    // https://modarchive.org/forums/index.php?topic=2709.0
    let key = "TICK_MULTIPLIER";
    let tick_multiplier = match env::var(key) {
        Ok(val) => val.parse::<u64>(),
        Err(_) => Ok(20),
    };

    player::Config {
        channels: chan_vec,
        interval: Duration::from_millis(6 * tick_multiplier.unwrap()),
    }
}
