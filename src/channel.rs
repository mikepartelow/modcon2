use crate::effect::Effect;

pub struct Channel {
    pub note: String,
    pub sample: u8,
    pub period: u16,
    pub effect: Effect,
}
