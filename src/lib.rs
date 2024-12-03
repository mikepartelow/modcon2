pub mod device;
pub mod hexdump;
pub mod note;
pub mod player;
pub mod sound;
pub mod track;

#[derive(Debug)]
pub enum Error {
    Sample(String),
}
