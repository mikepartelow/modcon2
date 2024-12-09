use crate::device::Device;
use crate::module::Module;
use crate::pattern::Pattern;
use crate::pcm;
use colored::Colorize;
use log::*;
use rodio::{OutputStream, Sink};
use std::str::FromStr;
use std::thread;
use tokio::time::{self, Duration};

pub struct Config {
    pub channels: Vec<usize>,
}

// FIXME: this should take a sample factory, get rid of play_module_notes
pub async fn play_module(module: &mut Module, cfg: Config) {
    let mut device = Device::new(module.num_channels);

    // FIXME: this (tempo) is set by the very first effect in the mod, and differs between thraddash.mod and knullakuk.mod
    let mut interval = time::interval(Duration::from_millis(20 * 6)); // 20 * 6 is not arbitrary: https://modarchive.org/forums/index.php?topic=2709.0

    for (i, &pidx) in module.pattern_table.iter().enumerate() {
        if i == 73 {
            // FIXME: hard-coded for knullakuk.mod
            // println!("FIXME: pidx==0 is not the way");
            device.stop_all();
            break; // FIXME: add a pattern table (aka order) iterator (or iterator generator) to module
        }
        let print_prefix = format!(
            "{:03}/{:03} P{:02}",
            i,
            module.pattern_table.len() - 1,
            pidx
        )
        .dimmed(); // FIXME: rustier than this

        let mut p_prevs = Vec::new();
        for _ in 0..4 {
            p_prevs.push(0);
        }

        let p: &mut Pattern = &mut module.patterns[pidx as usize];
        for (row, channels) in p.by_ref() {
            let mut row_str =
                String::from_str(&format!("R{:02}:", row)).expect("FIXME: expect is discouraged");

            for ch in &channels {
                // FIXME: refactor, unit test
                row_str += &format!(
                    "{}{} {} {} {}",
                    "|".red(),
                    ch.note.bright_yellow(),
                    if ch.sample == 0 {
                        "   ".cyan()
                    } else {
                        format!("{:02x}h", ch.sample).cyan()
                    },
                    if ch.period == 0 {
                        "   ".dimmed()
                    } else {
                        format!("{:03}", ch.period).white()
                    },
                    if ch.effect == 0 {
                        "     ".green()
                    } else {
                        format!("{:04x}h", ch.effect).green()
                    },
                );
            }

            println!("{} {}{}", print_prefix, row_str.blue(), "|".red());

            for chan_idx in 0..4 {
                let ch = &channels[chan_idx];
                let p_prev = p_prevs[chan_idx];
                let sample_idx: usize = if ch.sample == 0 {
                    0
                } else {
                    // FIXME: so ugly! the mod file has ch.sample=0 meaning "continue playing", and ch.sample=1 means module.samples[0]
                    // we could 1-index the samples array!
                    (ch.sample - 1) as usize
                };

                if ch.period == 0 && p_prev == 0 {
                    // no change from "not playing yet"
                } else if ch.period != 0 && ch.sample > 0 {
                    let period = if ch.period == 0 { p_prev } else { ch.period };
                    if ch.period != 0 {
                        // FIXME: refactor, remove magic numbers, and get the right magic numbers, this one isn't it
                        // FIXME: note 123456
                        let rate = (7159090.5 / (period as f32 * 2.0)) as u32;
                        let scaling_factor = module.samples[sample_idx].volume as f32 / 64.0;

                        // FIXME: do the scaling at sample load time, not here
                        let samples: Vec<u8> = module.samples[sample_idx]
                            .data
                            .iter()
                            .map(|b| (*b as f32 * scaling_factor) as u8)
                            .collect();

                        let ll = module.samples[sample_idx].loop_length;
                        let loop_it = ll > 1;

                        let new_source = pcm::Source::new(
                            module.samples[sample_idx].name.to_string(),
                            samples,
                            rate,
                            loop_it,
                            module.samples[sample_idx].loop_offset.into(),
                        )
                        .expect("FIXME");

                        // FIXME: make this more readable, like info!(module.samples[sample_idx]) calls some Sample method
                        if cfg.channels.contains(&chan_idx) {
                            info!(
                                "latching: {:02x} [{}] v{} f{} ll{} lo{} li{}",
                                sample_idx,
                                module.samples[sample_idx].name,
                                module.samples[sample_idx].volume,
                                module.samples[sample_idx].finetune,
                                module.samples[sample_idx].loop_length,
                                module.samples[sample_idx].loop_offset,
                                loop_it
                            );
                            device.latch(chan_idx, new_source);
                        }
                    } else if ch.sample == 0 {
                        println!("STOP!!");
                        device.stop(chan_idx);
                    }

                    p_prevs[chan_idx] = period;
                }
            }

            interval.tick().await; // FIXME: would a sleep be simpler? is any delay even necessary? does playing N ticks of queued audio provide the necessary delay?
            trace!("tick");
        }
    }
    debug!("exit1");
    device.wait();
    debug!("exit2");
}

pub fn play_samples(module: &mut Module, period: u8) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let _sink = Sink::try_new(&stream_handle).unwrap();

    for i in 0..module.samples.len() {
        let sample = &module.samples[i];
        println!("Sample {:02}: {}", i, sample.name);

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        // https://wiki.multimedia.cx/index.php/Protracker_Module
        // To get the actual note frequency, divide the Amiga base clock (70ns or 8363*428) by the period number.
        // https://www.aes.id.au/modformat.html
        // For PAL machines
        //   the clock rate is 7093789.2 Hz and for NTSC machines it is
        //   7159090.5 Hz. When the clock rate is divided by twice the
        //   period number for the pitch it will give the rate to send the
        //   data to the channel, eg. for a PAL machine sending a note at
        //   C2 (period 428), the rate is 7093789.2/856 ~= 8287.1369

        // FIXME: unify with note 123456
        let rate = (7093789.2 / ((period as u16 * 2) as f32)) as u32;

        let source = pcm::Source::new(sample.name.to_string(), sample.data.clone(), rate, false, 0)
            .expect("FIXME");

        sink.append(source);
        sink.sleep_until_end();

        let delay = time::Duration::from_millis(500);
        thread::sleep(delay);
    }
}
