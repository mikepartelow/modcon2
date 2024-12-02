use rodio::{OutputStream, Sink, Source};
use std::io::{Cursor, Read};
use std::io::{Seek, SeekFrom};
use std::time::{self, Duration};

#[derive(Clone)]
pub struct RawPcmSource {
    pub samples: Cursor<Vec<u8>>,
    pub sample_rate: u32,
}

impl RawPcmSource {
    pub fn advance(&mut self, bytes: u64) {
        self.samples.seek(SeekFrom::Current(bytes as i64)).unwrap();
    }
}

fn pop_front<T>(vec: &mut Vec<T>) -> Option<T> {
    if vec.is_empty() {
        return None;
    }
    Some(vec.remove(0))
}

impl Iterator for RawPcmSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // println!(">>> {}", self.samples.get_mut()[5]); // this seems correct. what?!

        let sample_byte = pop_front(self.samples.get_mut())?;

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
