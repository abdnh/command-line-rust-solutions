use std::{
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::PathBuf,
};

use clap::Parser;
use snafu::prelude::*;
use utils::LinesWithEol;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Input files
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,
    /// Print the first NUM lines of each file
    #[arg(
        short = 'n',
        long = "lines",
        value_name = "LINES",
        default_value_t = 10
    )]
    lines: usize,
    /// Print the first NUM bytes of each file
    #[arg(
        short = 'c',
        long = "bytes",
        value_name = "BYTES",
        conflicts_with = "lines"
    )]
    bytes: Option<usize>,
}

#[derive(Debug, Snafu)]
pub enum CliError {
    #[snafu(display("{}: {}", path.display(), source))]
    Io { source: io::Error, path: PathBuf },
}

pub type MyResult<T, R = CliError> = Result<T, R>;


pub fn run() -> MyResult<()> {
    let cli = Cli::parse();
    let mut files = cli.files;
    if files.is_empty() {
        files.push("-".into());
    }
    for (idx, path) in files.iter().enumerate() {
        let reader: Box<dyn BufRead>;
        let desc: String;
        if path.to_str().map(|p| p == "-").unwrap_or(false) {
            reader = Box::new(io::stdin().lock());
            desc = "standard input".into();
        } else {
            reader = Box::new(BufReader::new(
                File::open(&path).context(IoSnafu { path })?,
            ));
            desc = path.to_string_lossy().into();
        }
        if files.len() > 1 {
            println!("==> {desc} <==");
        }
        if let Some(bytes) = cli.bytes {
            let buf = reader.bytes().take(bytes).collect::<Result<Vec<u8>, io::Error>>().context(IoSnafu{path})?;
            print!("{}", String::from_utf8_lossy(buf.as_slice()));
        } else {
            for line in reader.lines_with_eol().take(cli.lines) {
                match line {
                    Ok(l) => print!("{}", l),
                    Err(error) => {
                        return Err(CliError::Io {
                            source: error,
                            path: path.clone(),
                        });
                    }
                }
            }
        }
        if idx != files.len() - 1 {
            println!();
        }
    }

    Ok(())
}
