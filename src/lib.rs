pub mod device;
pub mod module;
pub mod note;
pub mod pcm;
pub mod player;
pub mod sample;

#[derive(Debug)]
pub enum Error {
    Sample(String),
}
