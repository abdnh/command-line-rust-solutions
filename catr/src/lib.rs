use clap::Parser;
use snafu::prelude::*;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Files to concatenate
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,
    /// Number output lines
    #[arg(short = 'n', long = "number", default_value_t = false)]
    number_lines: bool,
    // Number nonempty output lines, overrides -n
    #[arg(short = 'b', long = "number-nonblank", default_value_t = false)]
    number_non_blank_lines: bool,
}

#[derive(Debug, Snafu)]
pub enum CliError {
    #[snafu(display("{}: {}", path.display(), source))]
    Io { source: io::Error, path: PathBuf },
}

type MyResult<T, E = CliError> = Result<T, E>;

pub fn run() -> MyResult<()> {
    let cli = Cli::parse();
    let mut files = cli.files.clone();
    if files.is_empty() {
        files.push("-".into());
    }
    let number_non_blank_lines = cli.number_non_blank_lines;
    // -b overrides -n
    let number_lines = cli.number_lines && !number_non_blank_lines;
    for path in files {
        let reader: Box<dyn BufRead>;
        if path.to_str().map(|p| p == "-").unwrap_or(false) {
            reader = Box::new(io::stdin().lock());
        } else {
            let f = File::open(&path).context(IoSnafu { path: path.clone() })?;
            reader = Box::new(BufReader::new(f));
        }
        let mut idx = 0;
        for line in reader.lines() {
            let line = line.context(IoSnafu { path: path.clone() })?;
            if number_lines || (number_non_blank_lines && !line.is_empty()) {
                println!("{:>6}\t{}", idx + 1, line);
                idx += 1;
            } else {
                println!("{}", line);
            }
        }
    }

    Ok(())
}
