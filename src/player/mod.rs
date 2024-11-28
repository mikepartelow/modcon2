use rodio::{OutputStream, Sink, Source};
use std::collections::VecDeque;
use std::io::{Cursor, Read};
use std::thread;
use std::time::{self, Duration};

use crate::{hexdump, track};

// https://github.com/Prezzodaman/pymod/blob/main/pymod/pymod.py
// https://www.ocf.berkeley.edu/~eek/index.html/tiny_examples/ptmod/

pub fn play(m: track::Module) -> Result<(), Box<dyn std::error::Error>> {
    // println!("{}", m);
    // println!("---");

    for sample_num in 2..3 {
        // let mut c = 0;
        // let mut text = Vec::with_capacity(hexdump::LINELEN);

        // 6 is a techno drum
        // f is a synth
        // let sample_num = 6;

        // next up:
        // - try the python pymod.py
        // - if it works, example sample[sample_num] in python
        // - compare to ours

        // hexdump::hex_dump_buffer(&m.samples[sample_num].data, &mut text, &mut c);

        let sample_data = &m.samples[sample_num].data;
        println!("  eh : {}", m.samples[sample_num].data[4]);
        // Convert signed 8-bit to unsigned 8-bit PCM format
        let unsigned_data: VecDeque<u8> = sample_data
            .iter()
            .map(|&b| ((b as u16 + 128) & 255) as u8)
            .collect();

        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        let source = RawPcmSource {
            samples: Cursor::new(unsigned_data),
            sample_rate: 44100, // Sample rate, adjust as needed
        };

        sink.append(source);
        sink.sleep_until_end();

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
        // println!(">>> {}", self.samples.get_mut()[5]); // this seems correct. what?!

        let sample_byte = self.samples.get_mut().pop_front()?;

        // FIXME: this doesn't match unsigned_data, why not?
        println!("->{}", sample_byte);

        let sample_byte = sample_byte as i16; // Convert to i16 for arithmetic
        let sample = (sample_byte - 128) as f32 / 128.0; // Perform the operation

        println!("  {}", sample);

        Some(sample)
    }
}

impl Source for RawPcmSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        2 // Mono
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
