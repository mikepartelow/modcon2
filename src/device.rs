use rodio::Source;
use rodio::{OutputStream, OutputStreamHandle, Sink};

pub struct Device {
    output_handle: OutputStreamHandle,
    _output_stream: OutputStream,
    sinks: Vec<Sink>,
}

impl Device {
    pub fn new(num_channels: usize) -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        let mut d = Self {
            _output_stream: stream,
            output_handle: stream_handle,
            sinks: Vec::with_capacity(num_channels),
        };

        for _i in 0..num_channels {
            d.sinks.push(Sink::try_new(&d.output_handle).unwrap());
        }
        d
    }

    pub fn latch(&mut self, channel_idx: usize, source: impl Source<Item = f32> + Send + 'static) {
        // fixme: bounds check
        self.sinks[channel_idx].stop();

        self.sinks[channel_idx] = Sink::try_new(&self.output_handle).unwrap();
        self.sinks[channel_idx].append(source);
    }

    pub fn stop(&mut self, channel_idx: usize) {
        // fixme: bounds check
        self.sinks[channel_idx].stop();
    }

    pub fn stop_all(&mut self) {
        for sink in &self.sinks {
            sink.stop()
        }
    }

    pub fn wait(&mut self) {
        for sink in &self.sinks {
            sink.sleep_until_end();
        }
    }
}
