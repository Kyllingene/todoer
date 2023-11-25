use std::{process::exit, io::ErrorKind};

use ignore::WalkBuilder;
use regex::Regex;
use sarge::prelude::*;

// TODO: add color

sarge! {
    Args,

    'h' help: bool,
    'i' ignore: bool,
    'H' hidden: bool,
    's' short: bool,
}

fn main() {
    let (args, remainder) = match Args::parse() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error (failed to parse arguments): {e}");
            exit(1);
        }
    };

    if args.help || remainder.len() > 1 {
        println!("todoer [options] [path]: Print all todo's found in a file tree");
        println!("Matches `BUG:`, `HACK:`, `TODO:`, `FIXME:`, and `XXX:`,");
        println!("after any common programming comment. Doesn't support mid-multiline.");
        println!("  -h |   --help : show this text");
        println!("  -i | --ignore : disregard .ignore/.gitignore");
        println!("  -H | --hidden : include hidden files/directories");
        println!("  -s |  --short : only print filenames, not paths");
        exit(0);
    }

    let path = remainder.get(0).map(String::as_str).unwrap_or("./");

    let re = Regex::new(
        "(#(\\[?)|//|;|--(\\[?)|/\\*+|\\{-|%(\\{?)|\\(\\*|<!--)+ *(BUG|HACK|TODO|FIXME|XXX) *:.*",
    )
    .unwrap();
    for entry in WalkBuilder::new(path)
        .hidden(!args.hidden)
        .ignore(!args.ignore)
        .build()
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
                        match e.kind() {
                            ErrorKind::InvalidData => {},
                            _ => eprintln!("error (failed to read file): {e}"),
                        }
                    }

                    Ok(data) => {
                        for (li, line) in data.lines().enumerate() {
                            for m in re.find_iter(line) {
                                let name = if args.short {
                                    entry.file_name().to_string_lossy()
                                } else {
                                    entry.path().to_string_lossy()
                                };
                                println!(
                                    " ({}:{}:{}) {}",
                                    name,
                                    li + 1,
                                    m.range().start + 1,
                                    m.as_str()
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
