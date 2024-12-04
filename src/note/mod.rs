pub fn get_freq(period: u16) -> Result<(f32, String), String> {
    if period == 0 {
        return Ok((1.0, String::from("   ")));
    }

    static NOTES: [&str; 12] = [
        "C-", "C#", "D-", "D#", "E-", "F-", "F#", "G-", "G#", "A-", "A#", "B-",
    ];

    static FREQS: [f32; 12] = [
        4186.01, 4434.92, 4698.63, 4978.03, 5274.04, 5587.65, 5919.91, 6271.93, 6644.88, 7040.00,
        7458.62, 7902.13,
    ];

    // FIXME: frequency corrections

    //           C    C#   D    D#   E    F    F#   G    G#   A    A#   B
    static OCTAVE1: [u16; 12] = [856, 808, 762, 720, 678, 640, 604, 570, 538, 508, 480, 453];
    static OCTAVE2: [u16; 12] = [428, 404, 381, 360, 339, 320, 302, 285, 269, 254, 240, 226];
    static OCTAVE3: [u16; 12] = [214, 202, 190, 180, 170, 160, 151, 143, 135, 127, 120, 113];

    // FIXME: hashmap
    for (i, p) in OCTAVE1.iter().enumerate() {
        if period == *p {
            return Ok((FREQS[i], format!("{}4", NOTES[i])));
        }
    }

    for (i, p) in OCTAVE2.iter().enumerate() {
        if period == *p {
            return Ok((FREQS[i], format!("{}5", NOTES[i])));
        }
    }

    for (i, p) in OCTAVE3.iter().enumerate() {
        if period == *p {
            return Ok((FREQS[i], format!("{}6", NOTES[i])));
        }
    }

    Err(format!("unknown period: {}", period))
}
