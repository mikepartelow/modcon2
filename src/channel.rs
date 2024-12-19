#[derive(Debug)]

pub struct Effect {
    // FIXME: enum
    pub kind: u8,
    pub x: u8,
    pub y: u8,
}
pub struct Channel {
    pub note: String,
    pub sample: u8,
    pub period: u16,
    pub effect: Option<Effect>,
}
