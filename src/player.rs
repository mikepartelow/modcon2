use crate::module::Module;

use crate::pcm;
use crate::{device::Device, formatter::RowFormatter};
use log::*;
use rodio::{OutputStream, Sink};
use std::collections::HashSet;
use std::thread;
use tokio::time::{self, Duration};

pub struct Config {
    pub channels: Vec<usize>,
    pub interval: Duration,
}

pub async fn play_module(module: &mut Module, cfg: Config) -> HashSet<u8> {
    let mut device = Device::new(module.num_channels);
    let mut effects = HashSet::new();

    // FIXME: this (tempo) is set by the very first effect in the mod (and then potentially reset later)
    let mut interval = time::interval(cfg.interval);

    let mut rowfmt = RowFormatter::new(module);
    println!("{}", rowfmt.header());

    for (i, &pidx) in module.pattern_table.iter().enumerate() {
        rowfmt.set_prefix(i, pidx);

        for (row, channels) in module.patterns[pidx as usize].by_ref() {
            println!("{}", rowfmt.format_row(row, &channels));

            for (chan_idx, ch) in channels.iter().enumerate() {
                let sample_idx: usize = match ch.sample {
                    0 => 0,                        // ch.sample == 0 means "continue playing"
                    _ => (ch.sample - 1) as usize, // ch.sample > 0 refers to our 0-indexed Vec<Sample>
                };

                if ch.period == 0 {
                    continue; // avoid divide by zero when computing `rate`
                }

                let sample = match ch.sample {
                    0 => &module.samples[device.source_id(chan_idx)],
                    _ => &module.samples[sample_idx],
                };

                // effects:
                // yehat: 0, c, e, f (3+),
                // thraddash: 0, 2, c, e, f
                // mycon: 0, c, e, f
                //
                // hyperspace: 0, c, f - impl 0 next
                //
                // kk: 0, 1, 2, 3, 4, 6, a, c, d, e, f
                let effect = ((ch.effect >> 8) & 0xff) as u8;
                effects.insert(effect);

                // impl effect 0 - hyperspace!

                let volume_scaling_factor = match effect {
                    0xc => (ch.effect & 0xff) as f32,
                    _ => 64.0,
                } / 64.0;
                debug!("scaling factor: {:.6}", volume_scaling_factor);

                // Convert signed 8-bit to unsigned 8-bit PCM format
                // Scale by volume

                // FIXME: refactor, remove magic numbers, and get the right magic numbers, this one isn't it
                // FIXME: note 123456
                let rate: u32 = (7159090.5 / (ch.period as f32 * 2.0)) as u32;

                let new_source = pcm::Source::new(
                    module.samples[sample_idx].name.to_string(),
                    &sample
                        .data
                        .iter()
                        .map(|b| (*b * volume_scaling_factor))
                        .collect::<Vec<f32>>(),
                    rate,
                    sample.is_looped(),
                    sample.loop_offset.into(),
                )
                .expect("FIXME");

                // FIXME: make this more readable, like info!(sample) calls some Sample method
                if cfg.channels.contains(&chan_idx) {
                    info!(
                        "latching: {:02x} [{}] v{} f{} ll{} lo{} li{} sl{}",
                        sample_idx,
                        sample.name,
                        sample.volume,
                        sample.finetune,
                        sample.loop_length,
                        sample.loop_offset,
                        sample.is_looped(),
                        sample.data.len(),
                    );
                    device.latch(chan_idx, new_source, sample_idx);
                }
            }

            interval.tick().await; // FIXME: would a sleep be simpler? is any delay even necessary? does playing N ticks of queued audio provide the necessary delay?
            trace!("tick");
        }
    }

    debug!("exit1");
    device.stop_all();
    device.wait();
    debug!("exit2");

    effects
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

        let source =
            pcm::Source::new(sample.name.to_string(), &sample.data, rate, false, 0).expect("FIXME");

        sink.append(source);
        sink.sleep_until_end();

        let delay = time::Duration::from_millis(500);
        thread::sleep(delay);
    }
}
