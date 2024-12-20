use crate::channel::Channel;
use crate::effect::Effect;
use crate::note;

#[derive(Debug)]
pub struct Pattern {
    pub data: [u8; 1024],
    pub ptr: usize, // FIXME: make ptr private
}

impl Iterator for Pattern {
    type Item = (usize, Vec<Channel>); // FIXME: should probably not be a vector but a fixed length slice

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.data.len() {
            self.ptr = 0;
            return None;
        }

        let mut items = Vec::new();

        let row = self.ptr / (4 * 4); // FIXME: replace magic numbers

        // FIXME: replace magic numbers
        for i in (self.ptr..self.ptr + 4 * 4).step_by(4) {
            let channel: [u8; 4] = self.data[i..i + 4]
                .try_into()
                .expect("FIXME: use of expect is discouraged");

            // Combine the first and third byte to get the sample (wwwwyyyy)
            let sample = (channel[0] & 0xF0) | ((channel[2] & 0xF0) >> 4);
            //  // Combine the first and second byte to get the period (xxxxxxxxxxxx)
            let period = ((channel[0] & 0x0F) as u16) << 8 | (channel[1] as u16);
            //  // Combine the third and fourth byte to get the effect (zzzzzzzzzzzz)
            let effect = ((channel[2] & 0x0F) as u16) << 8 | (channel[3] as u16);

            let note = note::from_period(period).expect("FIXME: use of expect is discouraged");

            items.push(Channel {
                note,
                sample,
                period,
                effect: Effect::parse(effect),
            });
        }
        self.ptr += 4 * 4; // FIXME: replace magic numbers

        Some((row, items))
    }
}
