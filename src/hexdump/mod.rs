use std::{fs, io::Read};

pub const LINELEN: usize = 16;

fn is_printable_ascii(byte: u8) -> bool {
    (32..=126).contains(&byte)
}

pub fn hex_dump(filename: &str) {
    let mut file = fs::File::open(filename)
        .expect("FIXME: use of expect is generally discouraged, but there was an error");

    hex_dump_file(&mut file);
}

pub fn hex_dump_buffer(chunk: &Vec<u8>, text: &mut Vec<u8>, c: &mut usize) {
    for b in chunk {
        print!("0x{:02x?} ", b);
        text.push(*b);
        *c += 1;
        if *c > LINELEN {
            print_text(text, c);
        }
    }
}

fn hex_dump_file(file: &mut fs::File) {
    let chunk_size = 0x4000;

    // FIXME: make this an object
    let mut c = 0;
    let mut text = Vec::with_capacity(LINELEN);

    loop {
        let mut chunk = Vec::with_capacity(chunk_size);
        let n = file
            .by_ref()
            .take(chunk_size as u64)
            .read_to_end(&mut chunk)
            .expect("FIXME: use of expect is generally discouraged, but there was an error");

        if n == 0 {
            break;
        }

        hex_dump_buffer(&chunk, &mut text, &mut c);

        if n < chunk_size {
            break;
        }
    }
}

fn print_text(text: &mut Vec<u8>, c: &mut usize) {
    print!("  ");
    for ch in &*text {
        if is_printable_ascii(*ch) {
            print!("{}", *ch as char);
        } else {
            print!(".");
        }
    }
    *c = 0;
    println!();
    text.clear();
}
