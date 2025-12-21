use std::{io::BufRead, ops::Range, path::PathBuf};

use clap::{Args, Parser};
use snafu::{ResultExt, Snafu};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Input files
    #[arg(default_value = "-")]
    files: Vec<PathBuf>,
    #[command(flatten)]
    ranges: RangeArgs,
    /// Use DELIM instead of TAB for field delimiter
    #[arg(
        short = 'd',
        long = "delimiter",
        value_name = "DELIM",
        default_value_t = '\t'
    )]
    delimiter: char,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct RangeArgs {
    /// Select only these fields
    #[arg(short = 'f', long = "fields", group = "range")]
    fields: Option<String>,
    /// Select only these bytes
    #[arg(short = 'b', long = "bytes", group = "range")]
    bytes: Option<String>,
    /// Select only these characters
    #[arg(short = 'c', long = "chars", group = "range")]
    chars: Option<String>,
}

#[derive(Debug, Snafu)]
pub enum CliError {
    #[snafu(display("{}: {}", path.display(), source))]
    IoPath {
        source: std::io::Error,
        path: PathBuf,
    },
    #[snafu(display("illegal list value: \"{}\"", text))]
    InvalidPosition { text: String },
    #[snafu(display("illegal list value: \"{}\"", text))]
    PositionParse {
        source: std::num::ParseIntError,
        text: String,
    },
    #[snafu(display(
        "First number in range ({}) must be lower than second number ({})",
        start,
        end
    ))]
    InvalidStartEnd { start: usize, end: usize },
    #[snafu(display("{}: {}", path.display(), source))]
    Csv { source: csv::Error, path: PathBuf },
}

pub type CliResult<T> = Result<T, CliError>;

pub type PositionList = Vec<Range<usize>>;

// TODO: refactor
fn parse_pos(ranges: &str) -> CliResult<PositionList> {
    let mut positions: PositionList = vec![];
    for range in ranges.split(',') {
        let mut nums: Vec<usize> = vec![];
        for s in range.split('-') {
            if s.trim().starts_with("+") {
                return Err(CliError::InvalidPosition {
                    text: ranges.into(),
                });
            }
            let n: usize = s.parse().context(PositionParseSnafu {
                text: ranges.to_string(),
            })?;
            if n == 0 {
                return Err(CliError::InvalidPosition {
                    text: ranges.into(),
                });
            }
            nums.push(n);
        }
        if nums.len() > 2 {
            return Err(CliError::InvalidPosition {
                text: ranges.into(),
            });
        }
        if nums.len() == 2 && nums[0] >= nums[1] {
            return Err(CliError::InvalidStartEnd {
                start: nums[0],
                end: nums[1],
            });
        }
        let is_range = nums.len() == 2;
        if nums.len() == 1 {
            nums.push(nums[0] - 1);
            nums.reverse();
        }

        if is_range {
            nums[0] -= 1;
        }

        positions.push(Range {
            start: nums[0],
            end: nums[1],
        });
    }

    Ok(positions)
}

pub fn run() -> CliResult<()> {
    let cli = Cli::parse();

    for path in cli.files {
        let buffer = match utils::reader_from_path(path.clone()) {
            Ok(buffer) => buffer,
            Err(err) => {
                eprintln!("{}", CliError::IoPath { source: err, path });
                continue;
            }
        };
        if let Some(ref byte_ranges) = cli.ranges.bytes {
            let pos_list = parse_pos(byte_ranges)?;
            for line in buffer.lines() {
                let line = line.context(IoPathSnafu { path: path.clone() })?;
                for pos in pos_list.iter().cloned() {
                    print!("{}", String::from_utf8_lossy(&line.as_bytes()[pos]));
                }
                println!()
            }
        } else if let Some(ref char_ranges) = cli.ranges.chars {
            let pos_list = parse_pos(char_ranges)?;
            for line in buffer.lines() {
                let line = line.context(IoPathSnafu { path: path.clone() })?;
                for pos in pos_list.iter().cloned() {
                    print!(
                        "{}",
                        line.chars()
                            .skip(pos.start)
                            .take(pos.end - pos.start)
                            .collect::<String>()
                    );
                }
                println!()
            }
        } else if let Some(ref fields) = cli.ranges.fields {
            let pos_list = parse_pos(fields)?;
            let mut csv_reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .delimiter(cli.delimiter as u8)
                .from_reader(buffer);
            for result in csv_reader.records() {
                let record = result.context(CsvSnafu { path: path.clone() })?;
                let fields: Vec<&str> = record.iter().collect();
                for pos in pos_list.iter().cloned() {
                    print!("{}", fields[pos].join(&cli.delimiter.to_string()));
                }
                println!()
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod unit_tests {
    use super::parse_pos;

    #[test]
    fn test_parse_pos() {
        // The empty string is an error
        assert!(parse_pos("").is_err());

        // Zero is an error
        let res = parse_pos("0");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0\"",);

        let res = parse_pos("0-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"0-1\"",);

        // A leading "+" is an error
        let res = parse_pos("+1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1\"",);

        let res = parse_pos("+1-2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"+1-2\"",);

        let res = parse_pos("1-+2");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-+2\"",);

        // Any non-number is an error
        let res = parse_pos("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a\"",);

        let res = parse_pos("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1,a\"",);

        let res = parse_pos("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"1-a\"",);

        let res = parse_pos("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "illegal list value: \"a-1\"",);

        // Wonky ranges
        let res = parse_pos("-");
        assert!(res.is_err());

        let res = parse_pos(",");
        assert!(res.is_err());

        let res = parse_pos("1,");
        assert!(res.is_err());

        let res = parse_pos("1-");
        assert!(res.is_err());

        let res = parse_pos("1-1-1");
        assert!(res.is_err());

        let res = parse_pos("1-1-a");
        assert!(res.is_err());

        // First number must be less than second
        let res = parse_pos("1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (1) must be lower than second number (1)"
        );

        let res = parse_pos("2-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "First number in range (2) must be lower than second number (1)"
        );

        // All the following are acceptable
        let res = parse_pos("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1]);

        let res = parse_pos("1,3");
        assert!(res.is_ok());

        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 2..3]);

        let res = parse_pos("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..3]);

        let res = parse_pos("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..1, 6..7, 2..5]);

        let res = parse_pos("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..15, 18..20]);
    }
}
