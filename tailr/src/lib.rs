use std::fs::File;
use std::io::BufReader;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::PathBuf;

use clap::Parser;
use snafu::ResultExt;
use snafu::Snafu;
use std::io::Read;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Input files
    #[arg(required = true)]
    files: Vec<PathBuf>,
    /// Output the last BYTES bytes; or use -c +K to output  bytes  starting with the Kth of each file
    #[arg(short = 'c', long = "bytes", value_name = "BYTES")]
    bytes: Option<String>,
    /// Output the last LINES lines, instead of the last 10; or use -n +K to output starting with the Kth
    #[arg(
        short = 'n',
        long = "lines",
        value_name = "LINES",
        default_value = "10",
        conflicts_with = "bytes"
    )]
    lines: Option<String>,
    // Suppresses printing of headers when multiple files are being examined.
    #[arg(short = 'q', long = "quiet")]
    suppress_headers: bool,
}

#[derive(Snafu, Debug)]
pub enum CliError {
    #[snafu(display("{}", source))]
    Io { source: std::io::Error },
    #[snafu(display("{}: {}", path.display(), source))]
    IoPath {
        source: std::io::Error,
        path: PathBuf,
    },
    #[snafu(display("illegal {} count -- {}", if *is_bytes {"byte"} else {"line"}, position))]
    InvalidPosition { position: String, is_bytes: bool },
}

pub type CliResult<T> = std::result::Result<T, CliError>;

fn parse_position(mut text: &str) -> Option<(i64, bool)> {
    let from_start = if let Some(t) = text.strip_prefix("+") {
        text = t;
        true
    } else {
        false
    };
    let text = text.trim_start_matches('-');
    text.parse()
        .ok()
        .map(|n: i64| (if from_start { n } else { -n }, from_start))
}

pub fn run() -> CliResult<()> {
    let cli = Cli::parse();
    let (is_bytes, position_str) = if let Some(s) = cli.bytes {
        (true, s)
    } else if let Some(s) = cli.lines {
        (false, s)
    } else {
        (false, "".to_string())
    };
    let (mut position, from_start) =
        parse_position(&position_str).ok_or(CliError::InvalidPosition {
            position: position_str,
            is_bytes,
        })?;
    if position > 0 && from_start {
        position -= 1;
    }
    let should_print_headers = cli.files.len() > 1 && !cli.suppress_headers;
    for (file_idx, path) in cli.files.iter().enumerate() {
        let mut buffer = match File::open(path.clone()) {
            Ok(f) => BufReader::new(f),
            Err(err) => {
                eprintln!(
                    "{}",
                    CliError::IoPath {
                        source: err,
                        path: path.clone()
                    }
                );
                continue;
            }
        };
        if should_print_headers {
            println!("==> {} <==", path.display());
        }

        if !from_start {
            buffer.seek(SeekFrom::End(0)).context(IoSnafu {})?;
        }

        if is_bytes {
            if buffer.seek_relative(position).is_err() {
                let _ = buffer.seek(SeekFrom::Start(0));
            }
        } else if from_start {
            let max_lines = position.abs();
            let mut lines_num = 0;
            while lines_num < max_lines {
                if buffer.seek_relative(1).is_err() {
                    break;
                };
                let mut buf = [0; 1];
                if buffer.read_exact(&mut buf).is_err() {
                    break;
                };
                if buf[0] == b'\n' {
                    lines_num += 1;
                } else if buffer.seek_relative(-1).is_err() {
                    break;
                }
            }
        } else {
            let max_lines = position.abs();
            let mut lines_num = 0;
            while lines_num <= max_lines {
                if buffer.seek_relative(-1).is_err() {
                    break;
                };
                let mut buf = [0; 1];
                if buffer.read_exact(&mut buf).is_err() {
                    break;
                };
                let is_newline = buf[0] == b'\n';
                if is_newline {
                    lines_num += 1;
                }
                if !(is_newline && lines_num > max_lines) && buffer.seek_relative(-1).is_err() {
                    break;
                }
            }
        }
        let mut buf = vec![];
        buffer.read_to_end(&mut buf).context(IoSnafu {})?;
        print!("{}", String::from_utf8_lossy(&buf));

        if should_print_headers && file_idx != cli.files.len() - 1 {
            println!();
        }
    }

    Ok(())
}
