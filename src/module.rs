use crate::pattern::{Pattern};
use crate::sample::Sample;
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

impl Module {
    pub fn new(
        title: String,
        samples: Vec<Sample>,
        pattern_table: Vec<u8>,
        patterns: Vec<Pattern>,
    ) -> Self {
        Module {
            num_channels: 4,
            title,
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

pub fn read(mut file: File) -> io::Result<Module> {
    let title = read_title(&mut file)?;
    let (pattern_table, patterns, samples) = read_samples(&mut file)?;

    Ok(Module::new(title, samples, pattern_table, patterns))
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
const SAMPLE_HEADER_SIZE: usize = 22 + 2 + 1 + 1 + 2 + 2;

fn read_samples(file: &mut File) -> io::Result<(Vec<u8>, Vec<Pattern>, Vec<Sample>)> {
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
