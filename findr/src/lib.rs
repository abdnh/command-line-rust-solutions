use clap::{Parser, ValueEnum};
use regex::Regex;
use relative_path::RelativePath;
use snafu::Snafu;
use std::{
    io,
    path::{MAIN_SEPARATOR, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Snafu)]
pub enum CliError {
    Io {
        source: io::Error,
    },
    #[snafu(display("{}: {}", path.display(), source))]
    IoPath {
        path: PathBuf,
        source: io::Error,
    },
    #[snafu(display("Failed to decode path: {:?}", path))]
    PathDecode {
        path: PathBuf,
    },
    WalkDir {
        source: walkdir::Error,
        path: PathBuf,
    },
}

pub type CliResult<T = ()> = Result<T, CliError>;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, ValueEnum, Clone, Copy)]
enum EntryType {
    #[value(name = "d")]
    Dir,
    #[value(name = "f")]
    File,
    #[value(name = "l")]
    Link,
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    // Search paths
    paths: Vec<PathBuf>,
    // Types
    #[arg(short = 't', long = "type", value_enum, value_name = "TYPE")]
    types: Vec<EntryType>,
    // Names
    #[arg(short = 'n', long = "name", value_name = "NAME")]
    names: Vec<Regex>,
}

pub fn run() -> CliResult {
    let cli = Cli::parse();

    let current_dir = PathBuf::from(".");
    let mut paths = cli.paths;
    if paths.is_empty() {
        paths.push(current_dir.clone());
    }
    for path in paths {
        let walker = WalkDir::new(path.clone()).follow_links(true);
        for entry in walker {
            match entry {
                Err(err) => {
                    eprint!("{}: {}", path.display(), err);
                }
                Ok(entry) => {
                    let entry_type = {
                        if entry.file_type().is_file() {
                            Some(EntryType::File)
                        } else if entry.file_type().is_dir() {
                            Some(EntryType::Dir)
                        } else if entry.file_type().is_symlink() {
                            Some(EntryType::Link)
                        } else {
                            None
                        }
                    };
                    if entry_type.is_none() {
                        continue;
                    }
                    if let Some(entry_type) = entry_type {
                        if !cli.types.is_empty() && !cli.types.contains(&entry_type) {
                            continue;
                        }
                    }
                    // println!("{}",entry.path().to_string_lossy());
                    if !cli.names.is_empty()
                        && !cli
                            .names
                            .iter()
                            .any(|pattern| pattern.is_match(&entry.file_name().to_string_lossy()))
                    {
                        continue;
                    }

                    // Satify Windows tests that use mixed separators...
                    let path = RelativePath::new(entry.path().to_str().unwrap());
                    let parent = path
                        .parent()
                        .map(|p| p.as_str())
                        .unwrap_or("")
                        .replace(MAIN_SEPARATOR, "/");
                    if parent.is_empty() {
                        println!("{}", path.file_name().unwrap());
                    } else {
                        println!("{}/{}", parent, path.file_name().unwrap());
                    }
                }
            }
        }
    }

    Ok(())
}
