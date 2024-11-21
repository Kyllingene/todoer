use std::cell::LazyCell;
use std::io::ErrorKind;
use std::process::exit;

use const_format::formatcp;
use ignore::WalkBuilder;
use regex::Regex;
use sarge::prelude::*;

mod helpers;
use helpers::get_hash;

const FILE_STYLE: &str = "\x1b[38;5;12m";
const PATH_STYLE: &str = "\x1b[38;5;8m";
const ERROR_STYLE: &str = "\x1b[38;5;1m";
const DESTYLE: &str = "\x1b[0m";

sarge! {
    Args,

    'h' help: bool,
    'i' ignore: bool,
    'H' hidden: bool,
    #ok 'e' exclude: Vec<String>,
}

fn main() {
    let (args, remainder) = Args::parse().unwrap_or_else(|e| {
        eprintln!("Failed to parse arguments): {e}{DESTYLE}");
        exit(1);
    });

    if args.help || remainder.len() > 1 {
        println!("todoer [options] [path]: Print all todo's found in a file tree");
        println!("Matches `BUG`, `HACK`, `TODO`, `FIXME`, and `XXX`,");
        println!("after any common programming comment. Doesn't support mid-multiline.");
        println!("  -h |   --help : show this text");
        println!("  -i | --ignore : disregard .ignore/.gitignore");
        println!("  -H | --hidden : include hidden files/directories");
        return;
    }

    let path = remainder.first().map_or("./", String::as_str);

    let re = {
        const COMMENT: &str = join!(
            "|", "#(\\[?)", // # OR #[
            "//", ";", "--(\\[?)", // -- OR --[
            "/\\*+",    // /* OR /** (etc.)
            "\\{-",     // {-
            "%(\\{?)",  // % OR %{
            "\\(\\*",   // (*
            "<!--",
        );

        const SPEC: &str = "BUG|HACK|TODO|FIXME|XXX";

        formatcp!("({COMMENT})+ *({SPEC}).*")
    };
    let re = Regex::new(re).unwrap();

    let mut last_parent = 0;

    for entry in WalkBuilder::new(path)
        .hidden(!args.hidden)
        .ignore(!args.ignore)
        .build()
    {
        match entry {
            Err(e) => {
                eprintln!("{ERROR_STYLE}Error (while walking directory): {e}{DESTYLE}");
            }

            Ok(entry) => {
                if let Some(e) = entry.error() {
                    eprintln!("{ERROR_STYLE}Error (while handling entry): {e}{DESTYLE}");
                }

                if entry.path().is_dir() {
                    continue;
                }

                // only stringify the name once, and only if any results are
                // actually printed for that file
                let name = LazyCell::new(|| entry.file_name().to_string_lossy());

                match std::fs::read_to_string(entry.path()) {
                    Err(e) => {
                        if !matches!(e.kind(), ErrorKind::InvalidData) {
                            eprintln!("{ERROR_STYLE}Error (failed to read file): {e}{DESTYLE}");
                        }
                    }

                    Ok(data) => {
                        for (li, line) in data.lines().enumerate() {
                            for m in re.find_iter(line) {
                                if let Some(parent) = entry.path().parent() {
                                    let parent_hash = get_hash(parent);
                                    if last_parent != parent_hash {
                                        last_parent = parent_hash;
                                        println!("{PATH_STYLE}{}:", parent.display());
                                    }
                                }

                                println!(
                                    "  ({FILE_STYLE}{}:{}:{}{DESTYLE})    \t{}",
                                    &*name,
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
