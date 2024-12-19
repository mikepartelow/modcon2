use crate::Error;
use rodio;
use std::time::Duration;

#[derive(Clone)]
pub struct Source {
    pub name: String,

    loop_it: bool,
    loop_offset: usize,
    ptr: usize,
    rate: u32,
    samples: Vec<f32>,

    arp: bool,
    arp_ptr: usize,
}

impl Source {
    pub fn zero() -> Self {
        Source {
            loop_it: false,
            loop_offset: 0,
            name: "".to_string(),
            ptr: 0,
            rate: 0,
            samples: Vec::new(),
            arp: false,
            arp_ptr: 0,
        }
    }

    pub fn new(
        name: String,
        samples: &[f32],
        rate: u32,
        loop_it: bool,
        loop_offset: usize,
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
            name,
            ptr: 0,
            rate,
            samples: f32_samples,
            arp: arp,
            arp_ptr: 0,
        })
    }
}

impl Iterator for Source {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.samples.is_empty() {
            return None;
        }

        if self.ptr >= self.samples.len() {
            if self.loop_it {
                self.ptr = self.loop_offset;
            } else {
                return None;
            }
        }

        self.ptr += 1;

        let sample_value = if self.arp {
            // Arpeggio effect variables
            // let arpeggio_step_duration = self.rate as usize / 3; // Adjust as needed for your sample rate and desired speed
            let arpeggio_step_duration = 40;
            let arpeggio_pattern = [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
                7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            ]; // Semitone steps for a major chord: root, major third, perfect fifth

            let arpeggio_pattern = [0, 4, 7];

            // Calculate current arpeggio step
            let arpeggio_step = (self.ptr / arpeggio_step_duration) % arpeggio_pattern.len();

            self.arp_ptr += 1;
            if self.arp_ptr >= arpeggio_pattern.len() {
                self.arp_ptr = 0
            }
            let arpeggion_step = self.arp_ptr;

            // Apply arpeggio effect
            let arpeggio_offset = arpeggio_pattern[arpeggio_step] as f32;

            // let s = self.samples[self.ptr - 1] * (2.0f32).powf(arpeggio_offset / 12.0);
            // println!("{}, {}, {}", arpeggio_step, arpeggio_offset, s);
            // s
            let mf = [11084.0453125, 13964.151574803149, 16574.27336448598];
            self.samples[self.ptr - 1]
        } else {
            self.samples[self.ptr - 1]
        };

        Some(sample_value)
    }
}

// impl Iterator for Source {
//     type Item = f32;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.samples.is_empty() {
//             return None;
//         }
//         if self.ptr >= self.samples.len() {
//             if self.loop_it {
//                 // println!("FIXME: LOOP IT {}", self.name);
//                 // FIXME: after first full playthrough, loop only up to sample.loop_length
//                 self.ptr = self.loop_offset; // FIXME: validate this leap of faith
//             } else {
//                 return None;
//             }
//         };
//         self.ptr += 1;

//         Some(self.samples[self.ptr - 1])
//     }
// }

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
            periods[self.arp_ptr]
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
