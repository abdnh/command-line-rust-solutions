use std::{
    io::BufRead,
    path::{Path, PathBuf},
};

use clap::Parser;
use regex::{Regex, RegexBuilder};
use snafu::{ResultExt, Snafu};
use utils::LinesWithEol;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Pattern
    pattern: String,
    /// Input paths
    #[arg(default_value = "-")]
    paths: Vec<PathBuf>,
    /// Output lines that don't match the pattern
    #[arg(short = 'v', long = "invert-match")]
    invert_match: bool,
    /// Match pattern in a case-insensitive manner
    #[arg(short = 'i', long = "insensitive")]
    ignore_case: bool,
    /// Output the number of lines matched
    #[arg(short = 'c', long = "count")]
    count: bool,
    /// Match pattern in given directories recursively
    #[arg(short = 'r', long = "recursive")]
    recursive: bool,
}

#[derive(Snafu, Debug)]
pub enum CliError {
    Io {
        source: std::io::Error,
    },
    #[snafu(display("{}: {}", path.display(), source))]
    IoPath {
        source: std::io::Error,
        path: PathBuf,
    },
    Walkdir {
        source: walkdir::Error,
    },
    #[snafu(display("Invalid pattern \"{}\"", pattern))]
    Regex {
        source: regex::Error,
        pattern: String,
    },
}

pub type CliResult<T> = std::result::Result<T, CliError>;

fn print_file_matches<P: AsRef<Path>, B: BufRead>(
    path: P,
    buffer: B,
    pattern: &Regex,
    invert_match: bool,
    print_filename: bool,
    print_count: bool,
) -> CliResult<()> {
    let mut match_count: usize = 0;
    for line in buffer.lines_with_eol() {
        let line = line.context(IoSnafu {})?;
        if !invert_match && !pattern.is_match(&line) {
            continue;
        }
        if !print_count {
            if print_filename {
                print!("{}:", path.as_ref().display());
            }
            print!("{line}");
        }
        match_count += 1;
    }
    if print_count {
        if print_filename {
            print!("{}:", path.as_ref().display());
        }
        println!("{match_count}");
    }

    Ok(())
}

pub fn run() -> CliResult<()> {
    let cli = Cli::parse();
    let pattern = RegexBuilder::new(&cli.pattern)
        .case_insensitive(cli.ignore_case)
        .build()
        .context(RegexSnafu {
            pattern: cli.pattern,
        })?;

    for path in cli.paths.iter() {
        if cli.recursive && path.is_dir() {
            let walker = WalkDir::new(path);
            for entry in walker {
                let entry = entry.context(WalkdirSnafu {})?;
                if entry.file_type().is_file() {
                    let path = entry.path();
                    let buffer = utils::reader_from_path(path).context(IoPathSnafu { path })?;
                    print_file_matches(path, buffer, &pattern, cli.invert_match, true, cli.count)?;
                }
            }
        } else {
            if path.is_dir() {
                eprintln!("{} is a directory", path.display());
                continue;
            }
            match utils::reader_from_path(path).context(IoPathSnafu { path }) {
                Ok(buffer) => {
                    print_file_matches(
                        path,
                        buffer,
                        &pattern,
                        cli.invert_match,
                        cli.paths.len() > 1,
                        cli.count,
                    )?;
                }
                Err(err) => {
                    eprintln!("{}", err);
                    continue;
                }
            }
        }
    }

    Ok(())
}
