use crate::Error;
use rodio;
use std::time::Duration;

#[derive(Clone)]
pub struct Source {
    pub name: String,

    loop_it: bool,
    loop_offset: usize,
    loop_length: usize,
    ptr: usize,
    rate: u32,
    samples: Vec<f32>,

    arp: bool,
    sample_end: usize,
}

impl Source {
    pub fn zero() -> Self {
        Source {
            loop_it: false,
            loop_offset: 0,
            loop_length: 0,
            name: "".to_string(),
            ptr: 0,
            rate: 0,
            samples: Vec::new(),
            arp: false,
            sample_end: 1,
        }
    }

    pub fn new(
        name: String,
        samples: &[f32],
        rate: u32,
        loop_it: bool,
        loop_offset: usize,
        loop_length: usize,
        arp: bool,
    ) -> Result<Self, Error> {
        // FIXME: log a warning. it's weird but apparently not an error to have a 0-len sample. yehat has one.

        // if samples.is_empty() {
        //     return Err(Error::Sample("0 length sample".to_string()));
        // }

        let f32_samples = samples.iter().map(|b| *b / 128.0).collect();

        Ok(Source {
            loop_it,
            loop_offset,
            loop_length,
            name,
            ptr: 0,
            rate,
            samples: f32_samples,
            sample_end: samples.len(),
            arp: arp,
        })
    }
}

impl Iterator for Source {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.samples.is_empty() {
            return None;
        }
        if self.ptr >= self.sample_end {
            if self.loop_it {
                // Once the sample has
                //   been played all of the way through, it will loop if the repeat
                //   length is greater than one. It repeats by jumping to this
                //   position in the sample and playing for the repeat length, then
                //   jumping back to this position, and playing for the repeat
                //   length: https://www.aes.id.au/modformat.html
                // FIXME: validate these leaps of faith
                self.ptr = self.loop_offset;
                self.sample_end = self.loop_offset + self.loop_length;
            } else {
                return None;
            }
        };
        self.ptr += 1;

        Some(self.samples[self.ptr - 1])
    }
}

impl rodio::Source for Source {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        // FIXME: move to ctor
        // FIXME: not self.rate, self.period
        let bf = 8363.0 / self.rate as f32;
        let m3 = 8363.0 / (bf * (2.0f32).powf(4.0 / 12.0));
        let p5 = 8363.0 / (bf * (2.0f32).powf(7.0 / 12.0));
        let periods = [self.rate, m3 as u32, p5 as u32];

        // println!("{:?}", periods);
        let period = if self.arp {
            periods[self.ptr % 3] // FIXME: this probably gets hosed by looping
        } else {
            periods[0]
        };

        // println!("{},{}", self.arp, period);
        let rate: u32 = (7159090.5 / (period as f32 * 2.0)) as u32;
        // self.rate
        rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
