use crate::Error;
use rodio::Source;
use std::time::Duration;

#[derive(Clone)]
pub struct RawPcmSource {
    pub name: String,

    loop_it: bool,
    loop_offset: usize,
    ptr: usize,
    rate: u32,
    samples: Vec<f32>,
}

impl RawPcmSource {
    pub fn zero() -> Self {
        RawPcmSource {
            loop_it: false,
            loop_offset: 0,
            name: "".to_string(),
            ptr: 0,
            rate: 0,
            samples: Vec::new(),
        }
    }

    pub fn new(
        name: String,
        samples: Vec<u8>,
        rate: u32,
        loop_it: bool,
        loop_offset: usize,
    ) -> Result<Self, Error> {
        if samples.is_empty() {
            return Err(Error::Sample("0 length sample".to_string()));
        }

        let f32_samples = samples
            .iter()
            .map(|b| (*b as i16 - 128) as f32 / 128.0)
            .collect();

        Ok(RawPcmSource {
            loop_it: loop_it,
            loop_offset: loop_offset,
            name: name,
            ptr: 0,
            rate: rate,
            samples: f32_samples,
        })
    }
}

impl Iterator for RawPcmSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.samples.len() == 0 {
            return None;
        }
        if self.ptr >= self.samples.len() {
            if self.loop_it {
                // println!("FIXME: LOOP IT {}", self.name);
                // FIXME: after first full playthrough, loop only up to sample.loop_length
                self.ptr = self.loop_offset; // FIXME: validate this leap of faith
            } else {
                return None;
            }
        };
        self.ptr += 1;

        Some(self.samples[self.ptr - 1])
    }
}

impl Source for RawPcmSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}