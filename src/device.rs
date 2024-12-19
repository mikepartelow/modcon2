use rodio::{OutputStream, OutputStreamHandle};
use rodio::{Source, SpatialSink};

pub struct Device {
    output_handle: OutputStreamHandle,
    _output_stream: OutputStream,
    sinks: Vec<SpatialSink>,
    source_ids: Vec<usize>,
}

impl Device {
    pub fn new(num_channels: usize) -> Self {
        assert!(num_channels == 4); // FIXME: while true for now, it should not remain so

        let (_output_stream, output_handle) = OutputStream::try_default().unwrap();
        let mut sinks = Vec::with_capacity(num_channels);

        for chan_idx in 0..num_channels {
            sinks.push(make_spatial_sink(chan_idx, &output_handle));
        }

        Self {
            _output_stream,
            output_handle,
            sinks,
            source_ids: vec![0; num_channels],
        }
    }

    pub fn latch(
        &mut self,
        channel_idx: usize,
        source: impl Source<Item = f32> + Send + 'static,
        source_id: usize,
    ) {
        // fixme: bounds check
        self.sinks[channel_idx].stop(); // seems unnecessary in practice

        self.sinks[channel_idx] = make_spatial_sink(channel_idx, &self.output_handle);
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

fn make_spatial_sink(chan_idx: usize, output_handle: &OutputStreamHandle) -> SpatialSink {
    // FIXME: un-hardcode the magic numbers
    // FIXME: understand the magic numbers
    // FIXME: error handling, not unwrap
    match chan_idx {
        // Channels 1 and 4 are on the left
        // https://www.aes.id.au/modformat.html
        0 | 3 => SpatialSink::try_new(
            output_handle,
            [-1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        )
        .unwrap(),
        // channels 2 and 3 are on the right.
        1 | 2 => SpatialSink::try_new(
            output_handle,
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
        )
        .unwrap(),
        _ => panic!("FIXME"),
    }
}
