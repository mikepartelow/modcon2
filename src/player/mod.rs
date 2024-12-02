use crate::sound::RawPcmSource;
use crate::track;
use rodio::Source;
use rodio::{source::SineWave, OutputStream, Sink};
use std::io::Cursor;
use std::str::FromStr;
use std::thread;
use tokio::time::{self, Duration};

pub async fn play_module(module: &mut track::Module) {
    let mut interval = time::interval(Duration::from_millis(20 * 6)); // 20 * 6 is not arbitrary: https://modarchive.org/forums/index.php?topic=2709.0

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let mut channel_sinks = Vec::new();

    for _ in 0..4 {
        channel_sinks.push(Sink::try_new(&stream_handle).unwrap());
    }

    for (i, &pidx) in module.pattern_table.iter().enumerate() {
        if i == 73 {
            break; // FIXME: knulla-specific hack, replace with module.num_patterns
        }
        if i < 4 {
            continue;
        }

        // FIXME: iterate to m.num_patterns - the actual number of patterns, not 128
        let print_prefix = format!(
            "{:03}/{:03} P{:03} ",
            i,
            module.pattern_table.len() - 1,
            pidx
        ); // FIXME: rustier than this

        let mut f_prevs = Vec::new();
        for _ in 0..4 {
            f_prevs.push(0);
        }

        let p: &mut track::Pattern = &mut module.patterns[pidx as usize];
        while let Some((row, channels)) = p.next() {
            let mut row_str =
                String::from_str(&format!("R{:02}:", row)).expect("FIXME: expect is discouraged");

            for ch in &channels {
                row_str += &format!(
                    "|{} {:02x} {:04x} {:04x}",
                    ch.note, ch.sample, ch.period, ch.effect
                );
                row_str += &"|";
            }

            println!("{} {}", print_prefix, row_str);

            for chan_idx in 0..4 {
                let ch = &channels[chan_idx];
                let f_prev = f_prevs[chan_idx];
                let sink = &channel_sinks[chan_idx];

                if ch.period == 0 && f_prev == 0 {
                    // no change from "not playing yet"
                    // NOOP
                    println!("  NOOP");
                } else {
                    let f: u32 = if ch.period == 0 {
                        f_prev
                    } else {
                        (100000.0 / (ch.period as f32)) as u32
                    };
                    println!("  {} -> {}", ch.period, f);

                    // FIXME: memoize playing sample and playing note
                    let period = if ch.period != 0 { ch.period } else { 214 };

                    let sample_rate = 7093789.2 / ((period as u16 * 2) as f32);
                    let sample_data = module.samples[ch.sample as usize].data.clone();

                    let wave = RawPcmSource {
                        samples: Cursor::new(sample_data),
                        sample_rate: sample_rate as u32,
                    };

                    let duration_ms = 20 * 6;
                    if chan_idx == 1 || chan_idx == 2 {
                        sink.append(wave.take_duration(Duration::from_millis(duration_ms)));
                    }

                    f_prevs[chan_idx] = f;
                }
            }

            interval.tick().await; // FIXME: would a sleep be simpler? is any delay even necessary? does playing N ticks of queued audio provide the necessary delay?
        }
    }
    channel_sinks[0].sleep_until_end();
}

pub fn play_samples(module: &mut track::Module, period: u8) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink = Sink::try_new(&stream_handle).unwrap();

    for i in 0..module.samples.len() {
        let sample = &module.samples[i];
        println!(
            "Sample {:02}: {}",
            i,
            String::from_utf8_lossy(&sample.header.name).to_string()
        );

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

        let sample_rate = 7093789.2 / ((period as u16 * 2) as f32);

        let source = RawPcmSource {
            samples: Cursor::new(sample.data.clone()),
            sample_rate: sample_rate as u32,
        };

        sink.append(source);
        sink.sleep_until_end();

        let ten_millis = time::Duration::from_secs(1);
        thread::sleep(ten_millis);
    }
}
