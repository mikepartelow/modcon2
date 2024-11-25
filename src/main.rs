use filer::hexdump::hex_dump;
use filer::tracker;
use std::env;
use std::io;
use std::process;

// https://www.aes.id.au/modformat.html
// https://modarchive.org/index.php?request=view_by_moduleid&query=48107
// https://web.archive.org/web/20100921225940/http://io.debian.net/~tar/debian/xmp/xmp-2.7.1/docs/formats/Ultimate_Soundtracker-format.txt
// https://github.com/mikepartelow/rust-chess/tree/main/app/src

// Up Next:

// rename Module to TrackerModule
// tm = TrackerModule(filename)
// print(tm.title())
// for s in tm.samples():
//   print(s)

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <filename> <hexdump|module>", args[0]);
        process::exit(1);
    }

    let filename = &args[1];
    let command = &args[2];

    // FIXME: not like this (maybe use a library? stdlib?)
    if command == "hexdump" {
        hex_dump(filename);
    } else if command == "module" {
        match tracker::read_module(filename) {
            Ok(module) => println!("{}", module),
            Err(e) => eprintln!("Error reading {}: {}", filename, e),
        }
    } else {
        // FIXME: this prints ugly
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Custom error: Invalid command: {command}"),
        ));
    }

    Ok(())
}
