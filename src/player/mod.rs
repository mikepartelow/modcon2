use rodio::{OutputStream, Sink, Source};
use std::collections::VecDeque;
use std::io::Cursor;
use std::thread;
use std::time::{self, Duration};

use crate::track;

// https://github.com/Prezzodaman/pymod/blob/main/pymod/pymod.py
// https://www.ocf.berkeley.edu/~eek/index.html/tiny_examples/ptmod/
// https://github.com/cmatsuoka/tracker-history

pub fn play(m: &track::Module) -> Result<(), Box<dyn std::error::Error>> {
    for pidx in &m.pattern_table {
        // println!("{}", *pidx)
        play_pattern(m, *pidx as usize)?;
    }
    Ok(())
}

// FIXME: what's &'static
fn get_freq(period: u16) -> Result<f32, String> {
    if period == 0 {
        return Ok(1.0);
    }
    // notelist = ["C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-"]

    static FREQS: [f32; 12] = [
        4186.01, 4434.92, 4698.63, 4978.03, 5274.04, 5587.65, 5919.91, 6271.93, 6644.88, 7040.00,
        7458.62, 7902.13,
    ];

    // FIXME: frequency corrections

    //           C    C#   D    D#   E    F    F#   G    G#   A    A#   B
    static OCTAVE1: [u16; 12] = [856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453];
    static OCTAVE2: [u16; 12] = [428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226];
    static OCTAVE3: [u16; 12] = [214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113];

    // FIXME: hashmap
    for (i, p) in OCTAVE1.iter().enumerate() {
        if period == *p {
            return Ok(FREQS[i]);
        }
    }

    for (i, p) in OCTAVE2.iter().enumerate() {
        if period == *p {
            return Ok(FREQS[i]);
        }
    }

    for (i, p) in OCTAVE3.iter().enumerate() {
        if period == *p {
            return Ok(FREQS[i]);
        }
    }

    return Err(format!("unknown period: {}", period));
}

pub fn play_pattern(m: &track::Module, idx: usize) -> Result<(), Box<dyn std::error::Error>> {
    let p = &m.patterns[idx];
    // println!("pattern: {}", idx);

    for (i, division) in p.data.chunks(4 * 4).enumerate() {
        for (j, channel) in division.chunks(4).enumerate() {
            // Combine the first and third byte to get the sample (wwwwyyyy)
            let sample = (channel[0] & 0xF0) | ((channel[2] & 0xF0) >> 4);
            //  // Combine the first and second byte to get the period (xxxxxxxxxxxx)
            let period = ((channel[0] & 0x0F) as u16) << 8 | (channel[1] as u16);
            //  // Combine the third and fourth byte to get the effect (zzzzzzzzzzzz)
            let effect = ((channel[2] & 0x0F) as u16) << 8 | (channel[3] as u16);

            // println!("FIXME: print out sample, period, effect and compare to python");

            print!(
                "pattern: {} row: {:02x} channel: {:02x} sample: {:02x} period: {:04x}",
                idx, i, j, sample, period
            );

            // https://github.com/NardJ/ModTrack-for-Python/blob/master/modtrack/tracker.py#L204
            // https://github.com/NardJ/ModTrack-for-Python/blob/master/modtrack/tracker.py#L828
            // https://github.com/NardJ/ModTrack-for-Python/blob/master/modtrack/tracker.py#L909

            let freq = get_freq(period)?;
            // let freq = if period > 0 {
            //     (7093789 / (period * 2) as u32)
            // } else {
            //     0
            // };

            // if j == 0 {
            play_sample(m, &m.samples[sample as usize], freq)?;
            // }
        }
    }
    Ok(())
}

pub fn play_sample(
    m: &track::Module,
    sample: &track::Sample,
    freq: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let sample_data = &sample.data;

    // 130.81 = C3
    // https://github.com/NardJ/ModTrack-for-Python/blob/master/modtrack/tracker.py#L837
    let freq_adj = if freq == 1.0 { 1.0 } else { 130.81 / freq };
    println!(" freq: {:012.6} freq_adj: {:012.6}", freq, freq_adj);

    let unsigned_data: VecDeque<u8> = sample_data
        .iter()
        .map(|&b| ((((b as u16 + 128) & 255) as f32) * freq_adj) as u8)
        .collect();

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    let source = RawPcmSource {
        samples: Cursor::new(unsigned_data),
        // FIXME: how is this supposed to be determined? it's apparently NOT 44.1Khz for knulla
        sample_rate: (44100.0 * 0.36) as u32, // Sample rate, adjust as needed
    };

    sink.append(source);
    sink.sleep_until_end();

    Ok(())
}

pub fn play_samples(m: &track::Module) -> Result<(), Box<dyn std::error::Error>> {
    for sample in &m.samples {
        play_sample(m, &sample, 1.0)?;

        let ten_millis = time::Duration::from_secs(1);
        thread::sleep(ten_millis);
    }

    Ok(())
}

struct RawPcmSource {
    samples: Cursor<std::collections::VecDeque<u8>>,
    sample_rate: u32,
}

impl Iterator for RawPcmSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample_byte = self.samples.get_mut().pop_front()?;

        let sample_byte = sample_byte as i16; // Convert to i16 for arithmetic
        let sample = (sample_byte - 128) as f32 / 128.0; // Perform the operation

        Some(sample)
    }
}

impl Source for RawPcmSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1 // Mono
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
