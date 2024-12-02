use rodio::{OutputStream, Sink, Source};
use std::io::{Cursor, Read};
use std::io::{Seek, SeekFrom};
use std::time::{self, Duration};

#[derive(Clone)]
pub struct RawPcmSource {
    pub samples: Vec<u8>,
    pub sample_rate: u32,
    pub taken: u64,
    pub loop_it: bool,
    // FIXME: make ptr private and add a New() fn
    pub ptr: usize,
    pub name: String,
}

impl RawPcmSource {
    pub fn advance(&mut self, bytes: usize) {
        if self.samples.len() == 0 {
            return;
        }
        if self.ptr + bytes > self.samples.len() {
            if self.loop_it {
                self.ptr = (self.ptr + bytes) % self.samples.len();
            } else {
                self.ptr = self.samples.len();
            }
        } else {
            self.ptr += bytes;
        }
        println!(
            "    len: {} bytes: {} ptr: {} loop_it: {} name: {}",
            self.samples.len(),
            bytes,
            self.ptr,
            self.loop_it,
            self.name,
        );
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
                self.ptr = 0;
            } else {
                return None;
            }
        };
        let sample_byte = self.samples[self.ptr];

        let sample_byte = sample_byte as i16; // Convert to i16 for arithmetic
        let sample = (sample_byte - 128) as f32 / 128.0; // Perform the operation

        self.taken += 1;
        self.ptr += 1;

        Some(sample)
    }
}

impl Source for RawPcmSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        // 1 is correct here
        1 // Mono
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
