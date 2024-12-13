use std::fmt;

#[derive(Debug)]
pub struct Sample {
    pub name: String,
    pub length: u16,
    pub finetune: u8,
    pub volume: u8,
    pub loop_offset: u16,
    pub loop_length: u16,
    pub data: Vec<u8>,
}

impl fmt::Display for Sample {
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

impl Sample {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let end = bytes
            .iter()
            .position(|&byte| byte == 0)
            .unwrap_or(bytes.len());
        let valid_utf8 = String::from_utf8_lossy(&bytes[0..end]).to_string();

        Self {
            name: valid_utf8,
            // Sample length in words (1 word = 2 bytes).
            length: 2 * u16::from_be_bytes([bytes[22], bytes[23]]),
            finetune: bytes[24],
            volume: bytes[25],
            loop_offset: u16::from_be_bytes([bytes[26], bytes[27]]),
            loop_length: u16::from_be_bytes([bytes[28], bytes[29]]),

            data: Vec::new(),
        }
    }

    pub fn is_looped(&self) -> bool {
        self.loop_length > 1
    }
}
