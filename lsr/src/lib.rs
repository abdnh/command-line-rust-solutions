use chrono::DateTime;
use clap::Parser;
use snafu::{ResultExt, Snafu};
use std::{
    fs::{self, DirEntry, Metadata},
    io::{self, Error},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
use tabular::{Row, Table};
use users::{get_group_by_gid, get_user_by_uid};
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(default_value = ".")]
    paths: Vec<PathBuf>,
    // Show long listing
    #[arg(short = 'l', long = "long")]
    long: bool,
    // Show hidden files
    #[arg(short = 'a', long = "all")]
    all: bool,
}

#[derive(Snafu, Debug)]
pub enum CliError {
    #[snafu(display("{}: {}", path.display(), source))]
    IoPath { source: io::Error, path: PathBuf },

    #[snafu(display("{}", message))]
    Users { message: String },
}

pub type CliResult<T> = std::result::Result<T, CliError>;

fn get_file_type(metadata: &Metadata) -> char {
    let file_type = metadata.file_type();
    if file_type.is_symlink() {
        'l'
    } else if file_type.is_dir() {
        'd'
    } else {
        '-'
    }
}

fn get_permissions(mut mode: u32) -> String {
    let bits: String = (0..9)
        .map(|i| {
            let b = mode & 1;
            mode >>= 1;
            if b == 0 {
                return '-';
            }
            ['x', 'w', 'r'][i % 3]
        })
        .collect();

    bits.chars().rev().collect()
}

fn print_path_info<T: AsRef<Path>>(
    path: T,
    all: bool,
    metadata: Metadata,
    table: Option<&mut Table>,
    last_item: bool,
) -> CliResult<()> {
    let base_name = path
        .as_ref()
        .file_name()
        .map(|f| f.to_string_lossy())
        .unwrap_or_else(|| path.as_ref().to_string_lossy());
    if base_name.starts_with('.') && !all {
        return Ok(());
    }
    if let Some(table) = table {
        let user = get_user_by_uid(metadata.uid()).ok_or_else(|| CliError::Users {
            message: "couldn't get current user".into(),
        })?;
        let group = get_group_by_gid(metadata.gid()).ok_or_else(|| CliError::Users {
            message: "couldn't get current user group".into(),
        })?;
        let date = DateTime::from_timestamp_secs(metadata.mtime()).unwrap();
        table.add_row(
            Row::new()
                .with_cell(format!(
                    "{}{}",
                    get_file_type(&metadata),
                    get_permissions(metadata.mode())
                ))
                .with_cell(metadata.nlink())
                .with_cell(user.name().display())
                .with_cell(group.name().display())
                .with_cell(metadata.size())
                .with_cell(date.format("%Y %b %d").to_string())
                .with_cell(path.as_ref().display()),
        );
    } else {
        print!("{}", path.as_ref().display());
        if !last_item {
            print!("  ");
        }
    }

    Ok(())
}

fn create_table() -> Table {
    Table::new("{:<} {:>} {:>} {:>} {:>} {:>} {:<}")
}

pub fn run() -> CliResult<()> {
    let cli = Cli::parse();

    let paths = cli.paths;

    let (dirs, files): (Vec<_>, Vec<_>) = paths.iter().partition(|p| p.is_dir());

    let mut table = create_table();
    for (i, path) in files.iter().copied().enumerate() {
        match path.metadata().context(IoPathSnafu { path: path.clone() }) {
            Ok(metadata) => {
                print_path_info(
                    path,
                    true,
                    metadata,
                    cli.long.then_some(&mut table),
                    i == files.len() - 1,
                )?;
            }
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        }
    }
    if !files.is_empty() {
        println!("{}", table);
    }

    for path in dirs {
        if paths.len() > 1 {
            println!("{}:", path.display());
        }

        match fs::read_dir(path).context(IoPathSnafu { path: path.clone() }) {
            Ok(iter) => {
                let entries: Vec<Result<DirEntry, Error>> = iter.collect();
                let entries_num = entries.len();
                let mut table = create_table();
                for (i, entry) in entries.into_iter().enumerate() {
                    match entry.context(IoPathSnafu { path: path.clone() }) {
                        Ok(entry) => {
                            match entry.metadata().context(IoPathSnafu { path: entry.path() }) {
                                Ok(metadata) => {
                                    print_path_info(
                                        entry.path(),
                                        cli.all,
                                        metadata,
                                        cli.long.then_some(&mut table),
                                        i == entries_num - 1,
                                    )?;
                                }
                                Err(err) => {
                                    eprintln!("{}", err);
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            eprintln!("{}", err);
                            break;
                        }
                    }
                }
                println!("{}", table);
            }
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod unittests {
    use crate::*;

    #[test]
    fn test_permissions() {
        assert_eq!(get_permissions(0o644), "rw-r--r--".to_string());
    }
}
