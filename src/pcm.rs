use crate::{effect::Effect, sample, Error};
use rand::Rng;
use rodio;
use std::time::Duration;
pub struct Source {
    pub name: String,

    loop_it: bool,
    loop_offset: usize,
    loop_length: usize,
    ptr: usize,
    period: u16,
    samples: Vec<f32>,

    effect: Effect,
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
            period: 0,
            samples: Vec::new(),
            effect: Effect::zero(),
            sample_end: 1,
        }
    }

    pub fn new(
        name: String,
        samples: &[f32],
        period: u16,
        loop_it: bool,
        loop_offset: usize,
        loop_length: usize,
        effect: Effect,
    ) -> Result<Self, Error> {
        // it's weird but apparently not an error to have a 0-len sample. yehat has one.

        let f32_samples = samples
            .iter()
            .map(|b| (*b * effect.volume()) / 128.0)
            .collect();

        Ok(Source {
            loop_it,
            loop_offset,
            loop_length,
            name,
            ptr: 0,
            period,
            samples: f32_samples,
            sample_end: samples.len(),
            effect,
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
                // *2 because loop_length is specified in words
                self.sample_end = self.loop_offset + self.loop_length * 2;
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
        // FIXME: self.ptr % 3 isn't exactly correct when looping
        self.effect.arp(self.period, self.ptr % 3)
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
