use crate::sound::RawPcmSource;
use crate::track::{self};
use rodio::Source;
use rodio::{source::SineWave, OutputStream, OutputStreamHandle, Sink};
use std::process::Output;
use std::str::FromStr;
use std::thread;
use tokio::time::{self, Duration};

pub struct Device {
    stream: OutputStream,
    output: OutputStreamHandle,
    sinks: Vec<Sink>,
}

impl Device {
    pub const NUM_CHANNELS: usize = 4;

    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        let mut d = Self {
            stream: stream,
            output: stream_handle,
            sinks: Vec::with_capacity(Self::NUM_CHANNELS),
        };

        for _i in 0..Self::NUM_CHANNELS {
            d.sinks.push(Sink::try_new(&d.output).unwrap());
        }
        d
    }

    pub fn latch(&mut self, channel_idx: usize, source: impl Source<Item = f32> + Send + 'static) {
        // fixme: bounds check
        self.sinks[channel_idx].stop();

        self.sinks[channel_idx] = Sink::try_new(&self.output).unwrap();
        self.sinks[channel_idx].append(source);
    }

    pub fn play(&mut self) {
        self.sinks[0].sleep_until_end();
    }
}
