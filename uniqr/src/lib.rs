use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::{io, path::PathBuf};

use clap::Parser;
use snafu::{ResultExt, Snafu};
use utils::LinesWithEol;

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
}

pub type CliResult<T = ()> = Result<T, CliError>;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Input file
    #[arg(value_name = "FILE")]
    in_file: Option<PathBuf>,
    /// Output file
    #[arg(value_name = "FILE")]
    out_file: Option<PathBuf>,
    /// Precede each output line with the count of the number of times the line occurred in the input, followed by a single space
    #[arg(short = 'c', long = "count")]
    count: bool,
}

fn is_newline(c: char) -> bool {
    c == '\n' || c == '\r'
}

fn trim_newline<T: AsRef<str>>(s: &T) -> &str {
    s.as_ref().trim_end_matches(is_newline)
}

fn print_line_and_count<I: Write, T: AsRef<str> + Display>(
    buf: &mut I,
    line: T,
    count: Option<usize>,
) -> std::io::Result<()> {
    if let Some(count) = count {
        write!(buf, "{count:4} ")?;
    }
    write!(buf, "{line}")?;

    Ok(())
}

pub fn run() -> CliResult {
    let cli = Cli::parse();
    let in_buffer: Box<dyn BufRead> = match cli.in_file {
        Some(path) => {
            if path
                .to_str()
                .ok_or(CliError::PathDecode { path: path.clone() })?
                == "-"
            {
                Box::new(std::io::stdin().lock())
            } else {
                Box::new(BufReader::new(
                    File::open(path.clone()).context(IoPathSnafu { path })?,
                ))
            }
        }
        None => Box::new(std::io::stdin().lock()),
    };
    let mut out_buffer: Box<dyn Write> = match cli.out_file {
        Some(path) => Box::new(BufWriter::new(
            File::create(path.clone()).context(IoPathSnafu { path })?,
        )),
        None => Box::new(std::io::stdout().lock()),
    };

    let mut previous_line: Option<String> = None;
    let mut current_count: usize = 0;
    for line in in_buffer.lines_with_eol() {
        let line = line.context(IoSnafu {})?;
        if let Some(previous) = previous_line {
            if trim_newline(&previous) == trim_newline(&line) {
                current_count += 1;
            } else {
                print_line_and_count(
                    &mut out_buffer,
                    previous.clone(),
                    cli.count.then_some(current_count),
                )
                .context(IoSnafu {})?;
                current_count = 1;
            }
            previous_line = Some(previous);
        } else {
            current_count = 1;
        }
        // Preserve first occurrence of the line
        if current_count == 1 {
            previous_line = Some(line);
        }
    }
    if let Some(previous_line) = previous_line {
        print_line_and_count(
            &mut out_buffer,
            previous_line,
            cli.count.then_some(current_count),
        )
        .context(IoSnafu {})?;
    }

    Ok(())
}
