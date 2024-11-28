use std::fmt;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

use crate::hexdump::hex_dump_buffer;
use crate::hexdump::LINELEN;

// https://www.aes.id.au/modformat.html
// https://github.com/8bitbubsy/pt2-clone/?tab=readme-ov-file
// https://wiki.multimedia.cx/index.php/Protracker_Module

// TODO:
// - focus on knulla
// - play 1 sample
// - refactor/learn
// - play whole song
// - play other songs

pub struct Module {
    pub title: String,
    pub samples: Vec<Sample>,
}

pub struct SampleHeader {
    name: [u8; 22],
    length: u16,
    finetune: u8,
    volume: u8,
    loop_offset: u16,
    loop_length: u16,
}

pub struct Sample {
    pub header: SampleHeader,
    pub data: Vec<u8>,
}

impl SampleHeader {
    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            name: bytes[0..22].try_into().unwrap(),
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
        write!(f, "title: [{}]", self.title)?;
        for (i, s) in self.samples.iter().enumerate() {
            let name = String::from_utf8_lossy(&s.header.name).to_string();
            write!(f, "\n  sample {:02}: [{}]", i, name)?;
        }
        Ok(())
    }
}

pub fn read_module(filename: &str) -> io::Result<Module> {
    let mut file = File::open(filename)?;
    let title = read_title(&mut file)?;
    let samples = read_samples(&mut file)?;

    Ok(Module {
        title: title,
        samples: samples,
    })
}

fn read_title(file: &mut File) -> io::Result<String> {
    let mut buffer = vec![0; 20];
    file.read_exact(&mut buffer)?;
    Ok(String::from_utf8_lossy(&buffer).to_string())
}

// FIXME: there could be 15 in some versions, have to check for M.K. marker
const NUM_SAMPLES: usize = 31;

fn read_samples(file: &mut File) -> io::Result<Vec<Sample>> {
    let mut samples: Vec<Sample> = Vec::new();

    for _ in 1..=NUM_SAMPLES {
        let mut buffer = vec![0; std::mem::size_of::<SampleHeader>()];
        file.read_exact(&mut buffer)?;

        samples.push(Sample {
            header: SampleHeader::from_bytes(&buffer),
            data: Vec::new(),
        });
    }

    let mut buffer = vec![0, 2];
    file.read_exact(&mut buffer);
    // println!("buffie");
    // let mut c = 0;
    // let mut text = Vec::with_capacity(LINELEN);
    // hex_dump_buffer(&buffer, &mut text, &mut c);

    let mut pattern_table = vec![0; 128];
    file.read_exact(&mut pattern_table)?;

    // let mut c = 0;
    // let mut text = Vec::with_capacity(LINELEN);
    // hex_dump_buffer(&pattern_table, &mut text, &mut c);
    // println!("");

    // this works for shofixti and knulla but not supox
    let mut num_patterns: usize = 0;
    for pidx in pattern_table {
        if pidx as usize > num_patterns {
            num_patterns = pidx as usize;
        }
    }
    num_patterns = num_patterns + 1;
    // end of "this works for"

    println!("num_patterns: {}", num_patterns);

    let mut buffer: Vec<u8> = vec![0; 4];
    file.read_exact(&mut buffer)?;
    let mk = String::from_utf8_lossy(&buffer).to_string();

    // FIXME: actually, first read 15 samples, check for this stuff, then backtrack if we don't find it.
    if mk != "M.K." {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Error: missing expected magic marker M.K."),
        ));
    }

    // FIXME: read pattern data
    let _ = file.seek(SeekFrom::Current((num_patterns * 1024).try_into().unwrap()));

    for s in samples.iter_mut() {
        s.data = vec![0; s.header.length as usize];
        file.read_exact(&mut s.data)?;
    }

    // FIXME: determine expected size, then compare with expected
    println!("pos: {}", file.stream_position().unwrap());

    Ok(samples)
}
