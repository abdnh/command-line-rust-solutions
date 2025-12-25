use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use clap::Parser;
use rand::prelude::*;
use regex::{Regex, RegexBuilder};
use snafu::{ResultExt, Snafu};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Database files/directories
    #[arg(value_name = "FILE", required = true)]
    sources: Vec<PathBuf>,
    /// Print out all fortunes which match the basic regular expression
    #[arg(short = 'm', long = "pattern")]
    pattern: Option<String>,
    /// Ignore case for pattern
    #[arg(short = 'i', long = "insensitive")]
    ignore_case: bool,

    /// A seed for the random generator
    #[arg(short = 's', long = "seed")]
    seed: Option<u64>,
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
    #[snafu(display("Invalid --pattern \"{}\"", pattern))]
    Regex {
        source: regex::Error,
        pattern: String,
    },
    #[snafu(display("{}: {}", path.display(), source))]
    Walkdir {
        source: walkdir::Error,
        path: PathBuf,
    },
}

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
struct Fortunes {
    pub quotes: Vec<String>,
    rng: StdRng,
}

impl Fortunes {
    pub fn new(seed: Option<u64>) -> Self {
        Self {
            quotes: Vec::new(),
            rng: seed.map_or_else(StdRng::from_os_rng, StdRng::seed_from_u64),
        }
    }

    pub fn add(&mut self, quote: String) {
        self.quotes.push(quote);
    }

    pub fn select_random(&mut self) -> Option<&String> {
        self.quotes.choose(&mut self.rng)
    }

    pub fn print_last_n<T: AsRef<Path>>(&self, path: T, n: usize) {
        if n == 0 {
            return;
        }
        eprintln!("({})\n%", path.as_ref().file_name().unwrap().display());
        for quote in self.quotes.iter().skip(self.quotes.len() - n) {
            println!("{quote}\n%");
        }
    }
}

static FORTUNE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("(?ms)(.*?)^%$").unwrap());

fn read_fortunes<T: AsRef<Path>, B: BufRead>(
    path: T,
    buffer: &mut B,
    pattern: Option<&Regex>,
    fortunes: &mut Fortunes,
) -> CliResult<usize> {
    let mut contents = String::new();
    buffer.read_to_string(&mut contents).context(IoPathSnafu {
        path: path.as_ref(),
    })?;
    let mut matched = 0;
    for cap in FORTUNE_RE.captures_iter(&contents) {
        let fortune: String = cap.get(1).unwrap().as_str().trim().into();
        if let Some(p) = pattern {
            if p.is_match(&fortune) {
                fortunes.add(fortune);
                matched += 1;
            }
        } else {
            fortunes.add(fortune);
        }
    }
    Ok(matched)
}

fn get_buffer<T: AsRef<Path>>(path: T) -> CliResult<BufReader<File>> {
    let file = File::open(&path).context(IoPathSnafu {
        path: path.as_ref(),
    })?;

    Ok(BufReader::new(file))
}

pub fn run() -> CliResult<()> {
    let cli = Cli::parse();

    let pattern = if let Some(pattern) = cli.pattern {
        Some(
            &RegexBuilder::new(&pattern)
                .case_insensitive(cli.ignore_case)
                .build()
                .context(RegexSnafu { pattern })?,
        )
    } else {
        None
    };
    let mut fortunes = Fortunes::new(cli.seed);
    for path in cli.sources {
        if path.is_dir() {
            let walker = WalkDir::new(path.clone());
            for entry in walker {
                let entry = entry.context(WalkdirSnafu { path: path.clone() })?;
                if entry.file_type().is_file() {
                    // let mut buffer = match File::open(entry.path()) {
                    //     Ok(f) => BufReader::new(f),
                    //     Err(err) => {
                    //         eprintln!("{}", err);
                    //         continue;
                    //     }
                    // };
                    let mut buffer = get_buffer(entry.path())?;
                    // eprintln!("({})", entry.file_name().display());
                    let matched = read_fortunes(entry.path(), &mut buffer, pattern, &mut fortunes)?;
                    fortunes.print_last_n(entry.path(), matched);
                }
            }
        } else {
            let mut buffer = get_buffer(&path)?;
            // eprintln!("({})", path.file_name().unwrap().display());
            let matched = read_fortunes(&path, &mut buffer, pattern, &mut fortunes)?;
            fortunes.print_last_n(&path, matched);
        }
    }

    if pattern.is_none() {
        if let Some(fortune) = fortunes.select_random() {
            println!("{fortune}");
        } else {
            println!("No fortunes found");
        }
    }

    Ok(())
}
