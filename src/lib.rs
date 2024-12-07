pub mod device;
pub mod hexdump;
pub mod module;
pub mod note;
pub mod player;
pub mod sound;

#[derive(Debug)]
pub enum Error {
    Sample(String),
}
