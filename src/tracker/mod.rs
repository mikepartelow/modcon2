use std::fmt;
use std::fs::File;
use std::io::{self, Read};

// https://www.aes.id.au/modformat.html

pub struct Sample {
    name: [u8; 22],
    length: u16,
    finetune: u8,
    volume: u8,
    loop_offset: u16,
    loop_length: u16,
}

impl Sample {
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
pub struct Module {
    title: String,
    samples: Vec<Sample>,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "title: [{}]", self.title)?;
        for (i, s) in self.samples.iter().enumerate() {
            let name = String::from_utf8_lossy(&s.name).to_string();
            write!(f, "\n  sample {:02}: [{}]", i, name)?;
        }
        Ok(())
    }
}

pub fn read_module(filename: &str) -> io::Result<Module> {
    let mut file = File::open(filename)?;
    let title = read_title(&mut file)?;
    Ok(Module {
        title: title,
        samples: read_samples(&mut file)?,
    })
}

fn read_title(file: &mut File) -> io::Result<String> {
    let mut buffer = vec![0; 20];
    file.read_exact(&mut buffer)?;
    Ok(String::from_utf8_lossy(&buffer).to_string())
}

const NUM_SAMPLES: usize = 16; // there could be 31 in some versions, how to tell?

fn read_samples(file: &mut File) -> io::Result<Vec<Sample>> {
    let mut samples: Vec<Sample> = Vec::new();

    for _ in 1..=NUM_SAMPLES {
        let mut buffer = vec![0; std::mem::size_of::<Sample>()];
        file.read_exact(&mut buffer)?;

        samples.push(Sample::from_bytes(&buffer));
    }

    Ok(samples)
}
