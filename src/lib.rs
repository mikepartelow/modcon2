pub mod channel;
pub mod device;
pub mod effect;
pub mod formatter;
pub mod module;
pub mod note;
pub mod pattern;
pub mod pcm;
pub mod player;
pub mod sample;

#[derive(Debug)]
pub enum Error {
    Sample(String),
}
