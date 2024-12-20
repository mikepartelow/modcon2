use std::fmt;

use log::*;

// effects:
// yehat: 0, c, e, f (3+),
// thraddash: 0, 2, c, e, f
// mycon: 0, c, e, f
//
// hyperspace: 0, c, f - impl 0 next
//
// kk: 0, 1, 2, 3, 4, 6, a, c, d, e, f

#[derive(Clone, Copy)]
pub enum Kind {
    None,
    Arp,
    SetVolume,
}

impl Kind {
    pub fn parse(k: u8, xy: u8) -> Self {
        match k {
            0 => match xy {
                0 => Kind::None,
                _ => Kind::Arp,
            },
            0xc => Kind::SetVolume,
            _ => {
                warn!("cannot parse {} to Kind", k);
                // FIXME: eventually this should panic!
                Kind::None
            }
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Kind::Arp => "Arp",
                _ => "None",
            }
        )
    }
}

#[derive(Clone, Copy)]
pub struct Effect {
    pub kind: Kind,
    pub x: u8,
    pub y: u8,
    pub xy: u8,
}

impl Effect {
    pub fn zero() -> Self {
        Self {
            kind: Kind::None,
            x: 0,
            y: 0,
            xy: 0,
        }
    }
    pub fn parse(e: u16) -> Self {
        let x = ((e >> 4) & 0xf) as u8;
        let y = (e & 0xf) as u8;
        let xy = (e & 0xff) as u8;

        Self {
            kind: Kind::parse(((e >> 8) & 0xff) as u8, xy),
            x,
            y,
            xy,
        }
    }

    pub fn pack(&self) -> u16 {
        let mut u = 0;

        u |= (self.kind as u16) << 8;
        u |= (self.x as u16) << 4;
        u |= self.y as u16;

        u
    }

    // the effects
    pub fn arp(&self, period: u16, idx: usize) -> u32 {
        let period = match self.kind {
            Kind::Arp => {
                // FIXME: move to ctor
                let bf = 8363.0 / period as f32;
                let m3 = (8363.0 / (bf * (2.0f32).powf((self.x as f32) / 12.0))) as u16;
                let p5 = (8363.0 / (bf * (2.0f32).powf((self.y as f32) / 12.0))) as u16;

                let periods = [period, m3, p5];

                periods[idx]
            }
            _ => period,
        };

        assert!(period != 0);

        // // FIXME: what is this magic number?
        (7159090.5 / (period as f32 * 2.0)) as u32
    }

    pub fn volume(&self) -> f32 {
        let v = match self.kind {
            Kind::SetVolume => self.xy as f32 / 64.0,
            _ => 1.0,
        };

        v
    }
}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.kind, self.xy)
    }
}
