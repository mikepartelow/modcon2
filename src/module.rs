use crate::pattern::{self, Pattern};
use crate::sample::Sample;
use log::*;
use std::fmt;
use std::io::Read;
use std::io::{self, Seek, SeekFrom};

#[derive(Debug)]
pub struct Module {
    pub num_channels: usize,
    pub title: String,
    pub num_positions: usize,
    pub pattern_table: Vec<u8>,
    pub patterns: Vec<Pattern>,
    pub samples: Vec<Sample>,
}

impl Module {
    pub fn new(
        title: String,
        samples: Vec<Sample>,
        num_positions: usize,
        pattern_table: Vec<u8>,
        patterns: Vec<Pattern>,
    ) -> Self {
        Module {
            num_channels: 4,
            title,
            num_positions,
            samples,
            pattern_table,
            patterns,
        }
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "title: [{}] ({})", self.title, self.title.len())?;
        for (i, s) in self.samples.iter().enumerate() {
            write!(f, "\n  sample {:02}: [{}]", i, s.name)?;
        }
        Ok(())
    }
}

pub fn read<R: Read + Seek>(mut file: R) -> io::Result<Module> {
    let title = read_title(&mut file)?;
    let samples: read_sample_headers(&mut file)?;

    let mut bytes = vec![0; 1];
    file.read_exact(&mut bytes)?;

    let num_positions = bytes[0] as usize;

    // "Historically set to 127, but can be safely ignored." : https://www.aes.id.au/modformat.html
    let mut bytes = vec![0; 1];
    file.read_exact(&mut bytes)?;

    let pattern_table = read_pattern_table(&mut file)?;

    // FIXME: determine expected size, then compare with expected

    let magic_four = read_magic_four(&mut file)?;

    let samples = read_sample_data(&mut file, &samples)?;

    let pos = file.stream_position()?;
    file.seek(SeekFrom::End(0))?;

    let filelen = file.stream_position()?;
    if pos != filelen {
        error!("Expected file length {} but got {}.", filelen, pos);
    }
    assert!(pos == filelen);

    Ok(Module::new(title, samples, num_positions, pattern_table, patterns))
}

fn read_title<R: Read>(file: &mut R) -> io::Result<String> {
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
const SAMPLE_HEADER_SIZE: usize = 22 + 2 + 1 + 1 + 2 + 2;

fn read_samples<R: Read>(file: &mut R) -> io::Result<(Vec<u8>, Vec<Pattern>, Vec<Sample>)> {
    let mut patterns: Vec<Pattern> = Vec::new();
    let mut samples: Vec<Sample> = Vec::new();

    for _ in 1..=NUM_SAMPLES {
        let mut buffer = vec![0; SAMPLE_HEADER_SIZE];
        file.read_exact(&mut buffer)?;

        samples.push(Sample::from_bytes(&buffer));
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
            // FIXME: Pattern::new(buffer.try_into().unwrap())
            data: buffer.try_into().unwrap(),
            ptr: 0, // FIXME: make ptr private
        })
    }

    for s in samples.iter_mut() {
        let mut data = vec![0; s.length as usize];

        file.read_exact(&mut data)?;

        // Convert signed 8-bit to unsigned 8-bit PCM format
        s.data = data
            .iter()
            .map(|&b| ((b as u16 + 128) & 255) as u8)
            .collect();
    }

    Ok((pattern_table, patterns, samples))
}
