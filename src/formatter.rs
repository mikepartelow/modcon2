use crate::module::Module;
use crate::pattern::Pattern;
use crate::pcm;
use crate::{channel::Channel, device::Device};
use colored::{ColoredString, Colorize};
use log::*;
use rodio::{OutputStream, Sink};
use std::str::FromStr;
use std::thread;
use tokio::time::{self, Duration};

pub struct RowFormatter {
    pattern_table_len: usize,
    prefix: String,
}

impl RowFormatter {
    pub fn new(module: &Module) -> Self {
        RowFormatter {
            pattern_table_len: module.pattern_table.len() - 1,
            prefix: "".to_string(),
        }
    }

    pub fn set_prefix(&mut self, row_idx: usize, pattern_idx: u8) {
        self.prefix = format!(
            "{:03}/{:03} P{:02}",
            row_idx, self.pattern_table_len, pattern_idx
        )
    }

    pub fn format_row(&mut self, row: usize, channels: &[Channel]) -> String {
        let mut row_str =
            String::from_str(&format!("R{:02}:", row)).expect("FIXME: expect is discouraged");

        for ch in channels {
            row_str += &Self::format_channel(ch);
        }

        format!("{} {}{}", self.prefix.dimmed(), row_str.blue(), "|".red())
    }

    fn format_channel(ch: &Channel) -> String {
        format!(
            "{}{} {} {} {}",
            "|".red(),
            ch.note.bright_yellow(),
            if ch.sample == 0 {
                "   ".cyan()
            } else {
                format!("{:02x}h", ch.sample).cyan()
            },
            if ch.period == 0 {
                "   ".dimmed()
            } else {
                format!("{:03}", ch.period).white()
            },
            if ch.effect == 0 {
                "     ".green()
            } else {
                format!("{:04x}h", ch.effect).green()
            },
        )
    }
}