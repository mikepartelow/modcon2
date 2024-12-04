use crate::device::Device;

use crate::sound::RawPcmSource;
use crate::track::{self};
use rodio::Source;
use rodio::{source::SineWave, OutputStream, Sink};
use std::str::FromStr;
use std::thread;
use tokio::time::{self, sleep, Duration};

// FIXME: this should take a sample factory, get rid of play_module_notes
pub async fn play_module(module: &mut track::Module) {
    let mut device = Device::new(module.num_channels);

    let mut interval = time::interval(Duration::from_millis(20)); // 20 * 6 is not arbitrary: https://modarchive.org/forums/index.php?topic=2709.0

    for (i, &pidx) in module.pattern_table.iter().enumerate() {
        if i == 73 {
            // FIXME!
            // println!("FIXME: pidx==0 is not the way");
            device.stop_all();
            break; // FIXME: add a pattern table (aka order) iterator (or iterator generator) to module
        }
        // println!("!!!!!! i: {} pattern: {}", i, pidx);
        let print_prefix = format!(
            "{:03}/{:03} P{:03} ",
            i,
            module.pattern_table.len() - 1,
            pidx
        ); // FIXME: rustier than this

        let mut p_prevs = Vec::new();
        for _ in 0..4 {
            p_prevs.push(0);
        }

        let p: &mut track::Pattern = &mut module.patterns[pidx as usize];
        while let Some((row, channels)) = p.next() {
            // FIXME: why not make this an iterator for consistency?
            let mut row_str =
                String::from_str(&format!("R{:02}:", row)).expect("FIXME: expect is discouraged");

            for ch in &channels {
                row_str += &format!(
                    "|{} {:02x} {:04x} {:04x}",
                    ch.note, ch.sample, ch.period, ch.effect
                );
            }
            row_str += &"|";
            println!("{} {}", print_prefix, row_str);

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

                // println!(
                //     "  chan_idx: {} ch.period: {} p_prev: {} ch.sample: {}",
                //     chan_idx, ch.period, p_prev, ch.sample
                // );

                if ch.period == 0 && p_prev == 0 {
                    // no change from "not playing yet"
                    // NOOP
                    // println!("NOOP");
                } else if ch.period != 0 && ch.sample > 0 {
                    let period = if ch.period == 0 { p_prev } else { ch.period };
                    if ch.period != 0 {
                        let rate = (7093789.2 / ((period as u16 * 2) as f32)) as u32;
                        let samples = &module.samples[sample_idx].data; // FIXME: what??

                        let new_source = RawPcmSource::new(
                            module.samples[sample_idx as usize].header.name.to_string(),
                            samples.to_vec(), // FIXME: what??
                            rate,
                            module.samples[sample_idx as usize].header.loop_offset != 1,
                            module.samples[sample_idx as usize]
                                .header
                                .loop_offset
                                .into(),
                        )
                        .expect("FIXME");
                        // println!(
                        //     "this guy: {}",
                        //     module.samples[sample_idx as usize].header.loop_offset != 1
                        // );
                        device.latch(chan_idx, new_source);
                    } else if ch.sample == 0 {
                        println!("STOP!!");
                        device.stop(chan_idx);
                    }

                    p_prevs[chan_idx] = period;
                }
            }

            interval.tick().await; // FIXME: would a sleep be simpler? is any delay even necessary? does playing N ticks of queued audio provide the necessary delay?
                                   // println!("    tick");
        }
    }
    println!("exit1");
    device.wait();
    // FIXME: we never get to exit2
    println!("exit2");
}

pub fn play_samples(module: &mut track::Module, period: u8) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink = Sink::try_new(&stream_handle).unwrap();

    for i in 0..module.samples.len() {
        let sample = &module.samples[i];
        println!("Sample {:02}: {}", i, sample.header.name);

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

        let rate = (7093789.2 / ((period as u16 * 2) as f32)) as u32;

        let source = RawPcmSource::new(
            sample.header.name.to_string(),
            sample.data.clone(),
            rate,
            false,
            0,
        )
        .expect("FIXME");

        sink.append(source);
        sink.sleep_until_end();

        let delay = time::Duration::from_millis(500);
        thread::sleep(delay);
    }
}