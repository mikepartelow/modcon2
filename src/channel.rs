#[derive(Debug)]
pub struct Channel {
    pub note: String,
    pub sample: u8,
    pub period: u16,
    pub effect: u16,
}
