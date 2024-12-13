use core::num;

use rodio::{OutputStream, OutputStreamHandle, Sink};
use rodio::{Source, SpatialSink};

pub struct Device {
    output_handle: OutputStreamHandle,
    _output_stream: OutputStream,
    sinks: Vec<SpatialSink>,
    source_ids: Vec<usize>,
}

impl Device {
    pub fn new(num_channels: usize) -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let mut sinks = Vec::with_capacity(num_channels);

        for i in 0..num_channels {
            // d.sinks.push(Sink::try_new(&d.output_handle).unwrap());
            let sink = if i == 0 || i == 3 {
                SpatialSink::try_new(
                    &stream_handle,
                    [-1.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                )
                .unwrap()
            } else {
                SpatialSink::try_new(
                    &stream_handle,
                    [1.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                )
                .unwrap()
            };
            sinks.push(sink);
        }

        let mut d = Self {
            _output_stream: stream,
            output_handle: stream_handle,
            sinks: sinks,
            source_ids: vec![0; num_channels],
        };

        d
    }

    pub fn latch(
        &mut self,
        channel_idx: usize,
        source: impl Source<Item = f32> + Send + 'static,
        source_id: usize,
    ) {
        // fixme: bounds check
        self.sinks[channel_idx].stop(); // seems unnecessary

        // self.sinks[channel_idx] = Sink::try_new(&self.output_handle).unwrap();
        // FIXME: de-dup this code versus new()
        // FIXME: un-hardcode the magic numbers
        // FIXME: assert num_channels == 4 or support arbitrary num_channels
        let sink = if channel_idx == 0 || channel_idx == 3 {
            SpatialSink::try_new(
                &self.output_handle,
                [-1.0, 0.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
            )
            .unwrap()
        } else {
            SpatialSink::try_new(
                &self.output_handle,
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
            )
            .unwrap()
        };
        self.sinks[channel_idx] = sink;
        self.sinks[channel_idx].append(source);

        self.source_ids[channel_idx] = source_id;
    }

    pub fn source_id(&self, channel_idx: usize) -> usize {
        // fixme: bounds check
        self.source_ids[channel_idx]
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
