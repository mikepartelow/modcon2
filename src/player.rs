use crate::effect::Effect;
use crate::module::Module;

use crate::sample::Sample;
use crate::{device::Device, formatter::RowFormatter};
use crate::{pcm};
use log::*;
use rodio::{OutputStream, Sink};
use std::collections::HashSet;
use std::thread::{self, sleep};
use tokio::time::{self, Duration};

pub struct Config {
    pub channels: Vec<usize>,
    pub interval: Duration,
}

pub async fn play_module(module: &mut Module, cfg: Config) -> HashSet<u8> {
    let mut device = Device::new(module.num_channels);
    let effects = HashSet::new();

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
                    continue;
                }

                let sample = match ch.sample {
                    0 => &module.samples[device.source_id(chan_idx)],
                    _ => &module.samples[sample_idx],
                };

                let new_source = pcm::Source::new(
                    module.samples[sample_idx].name.to_string(),
                    &sample.data,
                    ch.period,
                    sample.is_looped(),
                    sample.loop_offset.into(),
                    sample.loop_length.into(),
                    ch.effect,
                )
                .expect("FIXME");

                // FIXME: make this more readable, like info!(sample) calls some Sample method
                if cfg.channels.contains(&chan_idx) {
                    info!(
                        "latching: {:02x} [{}] v{} f{} ll{} lo{} li{} sl{} e{}",
                        sample_idx,
                        sample.name,
                        sample.volume,
                        sample.finetune,
                        sample.loop_length,
                        sample.loop_offset,
                        sample.is_looped(),
                        sample.data.len(),
                        ch.effect,
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

pub fn play_sample(sample: &Sample, period: u16, _arp: bool) {
    let mut device = Device::new(4);

    let source = pcm::Source::new(
        sample.name.to_string(),
        &sample.data,
        period,
        true,
        sample.loop_offset.into(),
        sample.loop_length.into(),
        Effect::zero(),
    )
    .expect("FIXME");

    device.latch(0, source, 1);

    sleep(Duration::from_secs(30));

    debug!("exit1");
    device.stop_all();
    device.wait();
    debug!("exit2");
}

pub fn play_samples(module: &mut Module, period: u16) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let _sink = Sink::try_new(&stream_handle).unwrap();

    for i in 0..module.samples.len() {
        let sample = &module.samples[i];
        println!("Sample {:02}: {}", i, sample.name);

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let source = pcm::Source::new(
            sample.name.to_string(),
            &sample.data,
            period,
            false,
            0,
            0,
            Effect::zero(),
        )
        .expect("FIXME");

        sink.append(source);
        sink.sleep_until_end();

        let delay = time::Duration::from_millis(500);
        thread::sleep(delay);
    }
}
