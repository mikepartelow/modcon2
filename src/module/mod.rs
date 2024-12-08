use crate::note;
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

#[derive(Debug)]
pub struct Module {
    pub num_channels: usize,
    pub title: String,
    pub pattern_table: Vec<u8>,
    pub patterns: Vec<Pattern>,
    pub samples: Vec<Sample>,
}

#[derive(Debug)]
pub struct Pattern {
    pub data: [u8; 1024],
    ptr: usize,
}

#[derive(Debug)]
pub struct Channel {
    pub note: String,
    pub sample: u8,
    pub period: u16,
    pub effect: u16,
}

impl Iterator for Pattern {
    type Item = (usize, Vec<Channel>); // FIXME: should probably not be a vector but a fixed length slice

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.data.len() {
            self.ptr = 0;
            return None;
        }

        let mut items = Vec::new();

        let row = self.ptr / (4 * 4); // FIXME: replace magic numbers

        // FIXME: replace magic numbers
        for i in (self.ptr..self.ptr + 4 * 4).step_by(4) {
            let channel: [u8; 4] = self.data[i..i + 4]
                .try_into()
                .expect("FIXME: use of expect is discouraged");

            // Combine the first and third byte to get the sample (wwwwyyyy)
            let sample = (channel[0] & 0xF0) | ((channel[2] & 0xF0) >> 4);
            //  // Combine the first and second byte to get the period (xxxxxxxxxxxx)
            let period = ((channel[0] & 0x0F) as u16) << 8 | (channel[1] as u16);
            //  // Combine the third and fourth byte to get the effect (zzzzzzzzzzzz)
            let effect = ((channel[2] & 0x0F) as u16) << 8 | (channel[3] as u16);

            let note = note::from_period(period).expect("FIXME: use of expect is discouraged");

            items.push(Channel {
                note,
                sample,
                period,
                effect,
            });
        }
        self.ptr += 4 * 4; // FIXME: replace magic numbers

        Some((row, items))
    }
}

#[derive(Debug)]
pub struct SampleHeader {
    pub name: String,
    pub length: u16,
    pub finetune: u8,
    pub volume: u8,
    pub loop_offset: u16,
    pub loop_length: u16,
}

impl fmt::Display for SampleHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "name: [{}] ({})", self.name, self.name.len())?;
        writeln!(f, "  length:      [{}]", self.length)?;
        writeln!(f, "  finetune:    [{}]", self.finetune)?;
        writeln!(f, "  volume:      [{}]", self.volume)?;
        writeln!(f, "  loop offset: [{}]", self.loop_offset)?;
        writeln!(f, "  loop length: [{}]", self.loop_length)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Sample {
    pub header: SampleHeader,
    pub data: Vec<u8>,
}

impl SampleHeader {
    pub const SIZE: usize = 22 + 2 + 1 + 1 + 2 + 2;

    fn from_bytes(bytes: &[u8]) -> Self {
        let end = bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(bytes.len());
        let valid_utf8 = String::from_utf8_lossy(&bytes[0..end]).to_string();

        Self {
            name: valid_utf8,
            // FIXME: why is this "2 *"" ?
            length: 2 * u16::from_be_bytes([bytes[22], bytes[23]]),
            finetune: bytes[24],
            volume: bytes[25],
            loop_offset: u16::from_be_bytes([bytes[26], bytes[27]]),
            loop_length: u16::from_be_bytes([bytes[28], bytes[29]]),
        }
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "title: [{}] ({})", self.title, self.title.len())?;
        for (i, s) in self.samples.iter().enumerate() {
            write!(f, "\n  sample {:02}: [{}]", i, s.header.name)?;
        }
        Ok(())
    }
}

pub fn read_module(filename: &str) -> io::Result<Module> {
    let mut file = File::open(filename)?;
    let title = read_title(&mut file)?;
    let (pattern_table, patterns, samples) = read_samples(&mut file)?;

    Ok(Module {
        num_channels: 4, // FIXME: magic number
        title,
        samples,
        pattern_table,
        patterns,
    })
}

fn read_title(file: &mut File) -> io::Result<String> {
    let mut bytes = vec![0; 20];
    file.read_exact(&mut bytes)?;
    let end = bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len());
    let valid_utf8 = String::from_utf8_lossy(&bytes[0..end]).to_string();

    Ok(valid_utf8)
}

// FIXME: there could be 15 in some versions, have to check for M.K. marker
const NUM_SAMPLES: usize = 31;

fn read_samples(file: &mut File) -> io::Result<(Vec<u8>, Vec<Pattern>, Vec<Sample>)> {
    let mut patterns: Vec<Pattern> = Vec::new();
    let mut samples: Vec<Sample> = Vec::new();

    for _ in 1..=NUM_SAMPLES {
        let mut buffer = vec![0; SampleHeader::SIZE];
        file.read_exact(&mut buffer)?;

        samples.push(Sample {
            header: SampleHeader::from_bytes(&buffer),
            data: Vec::new(),
        });
    }

    let mut buffer = vec![0, 2];
    let _ = file.read_exact(&mut buffer);

    let mut pattern_table = vec![0; 128];
    file.read_exact(&mut pattern_table)?;

    // this works for shofixti and knulla but not supox
    let mut num_patterns: usize = 0;
    for pidx in &pattern_table {
        if *pidx as usize > num_patterns {
            num_patterns = *pidx as usize;
        }
    }
    num_patterns += 1;
    // end of "this works for"

    let mut buffer: Vec<u8> = vec![0; 4];
    file.read_exact(&mut buffer)?;
    let mk = String::from_utf8_lossy(&buffer).to_string();

    // FIXME: actually, first read 15 samples, check for this stuff, then backtrack if we don't find it.
    if mk != "M.K." {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Error: missing expected magic marker M.K.".to_string(),
        ));
    }

    for _ in 0..num_patterns {
        let mut buffer: Vec<u8> = vec![0; 1024];
        file.read_exact(&mut buffer)?;
        patterns.push(Pattern {
            data: buffer.try_into().unwrap(),
            ptr: 0,
        })
    }

    for s in samples.iter_mut() {
        let mut data = vec![0; s.header.length as usize];

        file.read_exact(&mut data)?;

        // Convert signed 8-bit to unsigned 8-bit PCM format
        s.data = data
            .iter()
            .map(|&b| ((b as u16 + 128) & 255) as u8)
            .collect();
    }

    // FIXME: determine expected size, then compare with expected
    let pos = file.stream_position().unwrap();
    file.seek(SeekFrom::End(0))?;
    let filelen = file.stream_position().unwrap();
    // assert!(pos == filelen);
    if pos != filelen {
        println!("WARNING!!!!!11")
    }

    Ok((pattern_table, patterns, samples))
}
