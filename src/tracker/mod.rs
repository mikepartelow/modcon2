use std::fmt;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

// https://www.aes.id.au/modformat.html

pub struct Module {
    title: String,
    samples: Vec<Sample>,
    size: usize,
}

struct SampleHeader {
    name: [u8; 22],
    length: u16,
    finetune: u8,
    volume: u8,
    loop_offset: u16,
    loop_length: u16,
}

struct Sample {
    header: SampleHeader,
    data: [u8; 1024],
}

impl SampleHeader {
    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            name: bytes[0..22].try_into().unwrap(),
            length: u16::from_le_bytes([bytes[22], bytes[23]]),
            finetune: bytes[24],
            volume: bytes[25],
            loop_offset: u16::from_le_bytes([bytes[26], bytes[27]]),
            loop_length: u16::from_le_bytes([bytes[28], bytes[29]]),
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
        write!(f, "\nsize: {}b", self.size)?;
        Ok(())
    }
}

pub fn read_module(filename: &str) -> io::Result<Module> {
    let mut file = File::open(filename)?;
    let title = read_title(&mut file)?;
    let samples = read_samples(&mut file)?;
    let size = 20 + samples.len() * std::mem::size_of::<Sample>();

    Ok(Module {
        title: title,
        samples: samples,
        size: size,
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
            data: [0; 1024],
        });
    }

    // FIXME: actually, we need to read and parse this stuff
    file.seek(SeekFrom::Current(1 + 1 + 128))?;

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

    for s in samples.iter_mut() {
        file.read_exact(&mut s.data)?;
    }

    Ok(samples)
}
