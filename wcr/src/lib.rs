use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
};

use clap::Parser;
use snafu::Snafu;
use snafu::prelude::*;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Files to process
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,
    /// Print the byte counts
    #[arg(short = 'c', long = "bytes")]
    bytes: bool,
    /// Print the character counts
    #[arg(short = 'm', long = "chars", conflicts_with = "bytes")]
    chars: bool,
    /// Print the word counts
    #[arg(short = 'w', long = "words")]
    words: bool,
    /// Print the line counts
    #[arg(short = 'l', long = "lines")]
    lines: bool,
}

#[derive(Debug, Snafu)]
pub enum CliError {
    #[snafu(display("{}: {}", path.display(), source))]
    Io { source: io::Error, path: PathBuf },
}

pub type CliResult<T = ()> = Result<T, CliError>;

#[derive(Debug, PartialEq, Eq, Hash)]
enum Metric {
    Bytes,
    Chars,
    Words,
    Lines,
}

fn get_metric_counts(
    mut file: impl BufRead,
    metrics: &Vec<Metric>,
) -> io::Result<HashMap<&Metric, usize>> {
    let mut counts = HashMap::new();
    let mut line = String::new();
    loop {
        let read = file.read_line(&mut line)?;
        if read == 0 {
            break;
        }
        for metric in metrics {
            let count = match metric {
                Metric::Bytes => line.len(),
                Metric::Chars => line.chars().count(),
                Metric::Words => line.split_whitespace().count(),
                Metric::Lines => 1,
            };
            counts
                .entry(metric)
                .and_modify(|v| *v += count)
                .or_insert(count);
        }
        line.clear();
    }

    Ok(counts)
}

pub fn run() -> CliResult {
    let cli = Cli::parse();
    let mut included_metrics: Vec<Metric> = vec![];
    if cli.lines {
        included_metrics.push(Metric::Lines);
    }
    if cli.words {
        included_metrics.push(Metric::Words);
    }
    if cli.chars {
        included_metrics.push(Metric::Chars);
    }
    if cli.bytes {
        included_metrics.push(Metric::Bytes);
    }
    if included_metrics.is_empty() {
        // If no metric is explicitly specified, include lines, words, and bytes
        included_metrics.extend([Metric::Lines, Metric::Words, Metric::Bytes]);
    }

    let mut files = cli.files;
    if files.is_empty() {
        files.push("-".into());
    }
    let mut totals = vec![0; included_metrics.len()];

    for path in files.iter() {
        let reader: Box<dyn BufRead>;
        let is_stdin = path.to_str().map(|p| p == "-").unwrap_or(false);
        if is_stdin {
            reader = Box::new(io::stdin().lock());
        } else {
            let f = File::open(path).context(IoSnafu { path });
            match f {
                Err(err) => {
                    eprintln!("{err}");
                    continue;
                }
                Ok(f) => reader = Box::new(BufReader::new(f)),
            }
        }
        match get_metric_counts(reader, &included_metrics).context(IoSnafu { path }) {
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
            Ok(counts) => {
                for (i, metric) in included_metrics.iter().enumerate() {
                    let count = counts.get(metric).unwrap_or(&0);
                    totals[i] += count;
                    print!("{count:>8}");
                }
                if !is_stdin {
                    print!(" {}", path.display());
                }
                println!();
            }
        }
    }
    if files.len() > 1 {
        for total in totals {
            print!("{total:>8}");
        }
        println!(" total");
    }

    Ok(())
}
