use crate::pattern::Pattern;
use crate::sample::Sample;
use log::*;
use std::env;
use std::fmt::{self};
use std::io::Read;
use std::io::{self, Seek, SeekFrom};

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

pub fn read<R: Read + Seek>(mut file: R) -> io::Result<Module> {
    let title = read_title(&mut file)?;
    let mut samples = read_sample_headers(&mut file)?;

    // Number of song positions (ie. number of patterns played throughout the song).
    // second byte is "Historically set to 127, but can be safely ignored."
    // https://www.aes.id.au/modformat.html
    let mut bytes = vec![0; 2];
    file.read_exact(&mut bytes)?;

    let num_positions = bytes[0] as usize;

    let (pattern_table, num_patterns) = read_pattern_table(&mut file, num_positions)?;

    check_magic_four(&mut file)?;

    let patterns = read_patterns(&mut file, num_patterns)?;

    read_sample_data(&mut file, &mut samples)?;

    check_length(&mut file)?;

    Ok(Module::new(title, samples, pattern_table, patterns))
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

fn read_sample_headers<R: Read>(file: &mut R) -> io::Result<Vec<Sample>> {
    const NUM_SAMPLES: usize = 31;
    const SAMPLE_HEADER_SIZE: usize = 22 + 2 + 1 + 1 + 2 + 2;

    let mut samples: Vec<Sample> = Vec::new();

    for _ in 1..=NUM_SAMPLES {
        let mut buffer = vec![0; SAMPLE_HEADER_SIZE];
        file.read_exact(&mut buffer)?;

        samples.push(Sample::from_bytes(&buffer));
    }

    Ok(samples)
}

fn read_pattern_table<R: Read>(file: &mut R, num_positions: usize) -> io::Result<(Vec<u8>, usize)> {
    let mut pattern_table = vec![0; 128];
    file.read_exact(&mut pattern_table)?;

    // FIXME: this works for shofixti and knulla but not supox
    //        could the problem be solved by looping only until module::num_positions?
    let mut num_patterns_used: usize = 0;
    for pidx in &pattern_table {
        if *pidx as usize > num_patterns_used {
            num_patterns_used = *pidx as usize;
        }
    }
    num_patterns_used += 1;
    // end of "this works for"

    pattern_table.truncate(num_positions);

    Ok((pattern_table, num_patterns_used))
}

fn read_patterns<R: Read>(file: &mut R, num_patterns: usize) -> io::Result<Vec<Pattern>> {
    let mut patterns = Vec::new();

    info!("reading {} patterns", num_patterns);

    for _ in 0..num_patterns {
        let mut buffer: Vec<u8> = vec![0; 1024];
        file.read_exact(&mut buffer)?;
        patterns.push(Pattern {
            // FIXME: Pattern::new(buffer.try_into().unwrap())
            data: buffer.try_into().unwrap(),
            ptr: 0, // FIXME: make ptr private
        })
    }

    Ok(patterns)
}

fn check_magic_four<R: Read>(file: &mut R) -> io::Result<()> {
    let mut buffer: Vec<u8> = vec![0; 4];
    file.read_exact(&mut buffer)?;

    let mk = String::from_utf8_lossy(&buffer).to_string();

    // FIXME: actually, first read 15 samples, check for this stuff, then backtrack if we don't find it.
    if mk != "M.K." {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Error: missing expected magic marker 'M.K.', got '{}' instead.",
                mk
            ),
        ));
    }

    Ok(())
}

fn read_sample_data<R: Read>(file: &mut R, samples: &mut [Sample]) -> io::Result<()> {
    for s in samples.iter_mut() {
        let mut data = vec![0; s.length as usize];
        debug!("reading {} bytes of sample data", s.length);

        file.read_exact(&mut data)?;

        if !s.data.is_empty() {
            assert!(s.length > 1);
            data = data[2..data.len()].to_vec();
        }

        // FIXME: does this really actually do anything? It's meant to, but does it affect sound?
        let scaling_factor = s.volume as f32 / 64.0;
        debug!("scaling factor: {}", scaling_factor);

        // Convert signed 8-bit to unsigned 8-bit PCM format
        // Scale by volume
        s.data = data
            .iter()
            .map(|&b| ((b as u16 + 128) & 255) as u8)
            .map(|b| (b as f32 * scaling_factor) as u8)
            .collect();
    }

    Ok(())
}

fn check_length<R: Read + Seek>(mut file: R) -> io::Result<()> {
    match env::var("CHECK_LENGTH") {
        Ok(val) if val == "false" => return Ok(()),
        _ => {}
    };

    // FIXME: determine expected size, then compare with expected - don't just check that we are at EOF
    let pos = file.stream_position()?;
    file.seek(SeekFrom::End(0))?;

    let filelen = file.stream_position()?;
    if pos != filelen {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("Expected file length {} but got {}.", filelen, pos),
        ));
    }
    Ok(())
}
