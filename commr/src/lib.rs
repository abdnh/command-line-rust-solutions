use std::{cmp::Ordering, io::BufRead, path::PathBuf};

use clap::Parser;
use snafu::{ResultExt, Snafu};
use utils::LinesWithEol;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[arg(value_name = "FILE1")]
    file1: PathBuf,
    #[arg(value_name = "FILE2")]
    file2: PathBuf,
    /// Supress column 1 (lines unique to FILE1)
    #[arg(short = '1')]
    supress_1: bool,
    /// Supress column 2 (lines unique to FILE2)
    #[arg(short = '2')]
    supress_2: bool,
    /// Supress column 3 (lines that appear in both files)
    #[arg(short = '3')]
    supress_3: bool,
    /// Ignore case
    #[arg(short = 'i', long = "insensitive")]
    ignore_case: bool,
    // Delimiter
    #[arg(short = 'd', long = "delimiter", default_value_t = '\t')]
    delimiter: char,
}

#[derive(Snafu, Debug)]
pub enum CliError {
    #[snafu(display("{}", desc))]
    InvalidInput { desc: String },
    #[snafu(display("{}: {}", path.display(), source))]
    IoPath {
        source: std::io::Error,
        path: PathBuf,
    },
}

pub type CliResult<T> = std::result::Result<T, CliError>;

fn reader_from_path(path: PathBuf) -> CliResult<impl BufRead> {
    utils::reader_from_path(path.clone()).context(IoPathSnafu { path })
}

fn unwrap_line(path: &PathBuf, line: Option<std::io::Result<String>>) -> CliResult<Option<String>> {
    line.transpose().context(IoPathSnafu { path })
}

pub fn run() -> CliResult<()> {
    let cli = Cli::parse();

    if cli.file1 == cli.file2 && matches!(cli.file1.to_str(), Some("-")) {
        return Err(CliError::InvalidInput {
            desc: r#"Both input files cannot be STDIN ("-")"#.into(),
        });
    }

    let buffer1 = reader_from_path(cli.file1.clone())?;
    let buffer2 = reader_from_path(cli.file2.clone())?;
    let mut lines1 = buffer1.lines_with_eol();
    let mut lines2 = buffer2.lines_with_eol();

    let mut option1: Option<String> = None;
    let mut option2: Option<String> = None;
    loop {
        if option1.is_none() {
            option1 = unwrap_line(&cli.file1, lines1.next())?;
        }
        if option2.is_none() {
            option2 = unwrap_line(&cli.file2, lines2.next())?;
        }

        match (option1.clone(), option2.clone()) {
            (Some(line1), Some(line2)) => {
                let cmp_result = if cli.ignore_case {
                    &line1.to_lowercase().cmp(&line2.to_lowercase())
                } else {
                    &line1.cmp(&line2)
                };
                match cmp_result {
                    Ordering::Equal => {
                        if !cli.supress_3 {
                            if !cli.supress_1 {
                                print!("{}", cli.delimiter);
                            }
                            if !cli.supress_2 {
                                print!("{}", cli.delimiter);
                            }
                            print!("{line1}");
                        }
                        option1 = None;
                        option2 = None;
                    }
                    Ordering::Less => {
                        if !cli.supress_1 {
                            print!("{line1}");
                        }
                        option1 = None;
                    }
                    Ordering::Greater => {
                        if !cli.supress_2 {
                            if !cli.supress_1 {
                                print!("{}", cli.delimiter);
                            }
                            print!("{line2}");
                        }
                        option2 = None;
                    }
                }
            }
            (Some(line1), None) => {
                if !cli.supress_1 {
                    print!("{line1}");
                }
                option1 = None;
            }
            (None, Some(line2)) => {
                if !cli.supress_2 {
                    if !cli.supress_1 {
                        print!("{}", cli.delimiter);
                    }
                    print!("{line2}");
                }
                option2 = None;
            }
            (None, None) => break,
        }
    }

    Ok(())
}
