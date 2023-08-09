use std::process::exit;

use ignore::WalkBuilder;
use regex::Regex;
use sarge::prelude::*;

// TODO: add color

fn main() {
    let mut parser = ArgumentParser::new();
    parser.add(arg!(flag, both, 'h', "help"));
    parser.add(arg!(flag, both, 'i', "ignore"));
    parser.add(arg!(flag, both, 'H', "hidden"));

    let remainder = match parser.parse() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error (failed to parse arguments): {e}");
            exit(1);
        }
    };

    if get_flag!(parser, both, 'h', "help") || remainder.len() > 1 {
        println!(
            "{} [options] [path]: Print all todo's found in a file tree",
            parser.binary.unwrap_or_else(|| "todoer".to_string())
        );
        println!("Matches `BUG:`, `HACK:`, `TODO:`, `FIXME:`, and `XXX:`,");
        println!("after any common programming comment. Doesn't support multiline.");
        println!("  -h |   --help : show this text");
        println!("  -i | --ignore : disregard .ignore/.gitignore");
        println!("  -H | --hidden : include hidden files/directories");
        exit(0);
    }

    let path = remainder.get(0).map(String::as_str).unwrap_or("./");

    let re = Regex::new("(#|//|;;?|--) *(BUG|HACK|TODO|FIXME|XXX) *:.*").unwrap();
    for entry in WalkBuilder::new(path)
        .hidden(!get_flag!(parser, both, 'H', "hidden"))
        .ignore(!get_flag!(parser, both, 'i', "ignore")).build()
    {
        match entry {
            Err(e) => {
                eprintln!("error (while walking directory): {e}");
            }

            Ok(entry) => {
                if let Some(e) = entry.error() {
                    eprintln!("error (failed to parse ignore): {e}");
                }

                if entry.path().is_dir() {
                    continue;
                }

                match std::fs::read_to_string(entry.path()) {
                    Err(e) => {
                        eprintln!("error (failed to read file): {e}");
                    }

                    Ok(data) => {
                        for m in re.find_iter(&data) {
                            println!(" - {}", m.as_str());
                        }
                    }
                }

            }
        }
    }
}
