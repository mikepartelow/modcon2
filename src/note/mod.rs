pub fn from_period(period: u16) -> Result<(String), String> {
    if period == 0 {
        return Ok(String::from("   "));
    }

    static NOTES: [&str; 12] = [
        "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
    ];

    //           C    C#   D    D#   E    F    F#   G    G#   A    A#   B
    static OCTAVE1: [u16; 12] = [856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453];
    static OCTAVE2: [u16; 12] = [428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226];
    static OCTAVE3: [u16; 12] = [214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113];

    // FIXME: hashmap
    // FIXME: simplify this code
    // FIXME: probably need to return "close enough" notes, too
    for (i, p) in OCTAVE1.iter().enumerate() {
        if period == *p {
            return Ok(format!("{}4", NOTES[i]));
        }
    }

    for (i, p) in OCTAVE2.iter().enumerate() {
        if period == *p {
            return Ok(format!("{}5", NOTES[i]));
        }
    }

    for (i, p) in OCTAVE3.iter().enumerate() {
        if period == *p {
            return Ok(format!("{}6", NOTES[i]));
        }
    }

    Err(format!("unknown period: {}", period))
}
