use ansi_term::Style;
use chrono::Datelike;
use chrono::Local;
use chrono::Month;
use chrono::NaiveDate;
use chrono::Weekday;
use clap::Parser;
use itertools::Itertools;
use num_traits::cast::FromPrimitive;
use snafu::prelude::*;
use std::str::FromStr;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Show whole current year
    #[arg(short = 'y', long = "year", groups = ["g1", "g3"])]
    show_year: bool,
    /// Show given month of the current year
    #[arg(short = 'm', long = "month", value_name = "MONTH", value_parser = parse_month, groups = ["g1", "g2"])]
    month: Option<u32>,
    #[arg(value_name = "YEAR", value_parser = parse_year, groups = ["g3"])]
    year: Option<i32>,
}

#[derive(Debug, Snafu)]
pub enum CliError {
    #[snafu(display("{}", source))]
    Parse { source: std::num::ParseIntError },
    #[snafu(display("{}", message))]
    ParseWithMessage {
        source: std::num::ParseIntError,
        message: String,
    },
    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

pub type CliResult<T> = Result<T, CliError>;

fn parse_year(s: &str) -> CliResult<i32> {
    let year: i32 = s.parse().context(ParseSnafu {})?;
    if (1..=9999).contains(&year) {
        Ok(year)
    } else {
        whatever!(r#"{} is not in 1..=9999"#, year)
    }
}

fn parse_month(s: &str) -> CliResult<u32> {
    if let Ok(month) = Month::from_str(s) {
        return Ok(month as u32 + 1);
    };
    let month: u32 = s.parse().with_context(|_| ParseWithMessageSnafu {
        message: format!(r#"Invalid month "{}""#, s),
    })?;
    if (1..=12).contains(&month) {
        Ok(month)
    } else {
        whatever!(r#"month "{}" not in the range 1 through 12"#, month)
    }
}

fn get_month_name(month: u32) -> Option<String> {
    Month::from_u32(month).map(|m| {
        match m {
            Month::January => "January",
            Month::February => "February",
            Month::March => "March",
            Month::April => "April",
            Month::May => "May",
            Month::June => "June",
            Month::July => "July",
            Month::August => "August",
            Month::September => "September",
            Month::October => "October",
            Month::November => "November",
            Month::December => "December",
        }
        .into()
    })
}

fn weekday_to_ordinal(weekday: Weekday) -> u32 {
    match weekday {
        Weekday::Sun => 1,
        Weekday::Mon => 2,
        Weekday::Tue => 3,
        Weekday::Wed => 4,
        Weekday::Thu => 5,
        Weekday::Fri => 6,
        Weekday::Sat => 7,
    }
}

pub fn run() -> CliResult<()> {
    let cli = Cli::parse();

    let date = Local::now().date_naive();
    let year = cli.year.unwrap_or(date.year());
    let month = cli.month.or_else(|| {
        if cli.year.is_none() && !cli.show_year {
            return Some(date.month());
        }
        None
    });

    if let Some(m) = month {
        println!(
            "{:^20}  ",
            format!("{} {}", get_month_name(m).unwrap(), year)
        );
    } else {
        println!("{:>32}", year);
    }

    let today = Local::now().date_naive();
    let start = month.unwrap_or(1);
    let end = month.unwrap_or(12);
    for month_chunk in (start..=end).chunks(3).into_iter() {
        let month_chunk: Vec<u32> = month_chunk.collect();
        if month_chunk.len() > 1 {
            for i in month_chunk.iter().copied() {
                print!("{:^20}  ", get_month_name(i).unwrap());
            }
            println!();
        }
        for _ in month_chunk.iter() {
            print!("Su Mo Tu We Th Fr Sa  ")
        }
        println!();

        let mut current_days: Vec<u32> = vec![1; month_chunk.len()];
        for days_chunk in (1..=42).chunks(7).into_iter() {
            let days_chunk: Vec<u32> = days_chunk.collect();
            for (month_idx, month) in month_chunk.iter().copied().enumerate() {
                for day in days_chunk.iter().copied() {
                    if let Some(date) =
                        NaiveDate::from_ymd_opt(year, month, current_days[month_idx])
                    {
                        let ordinal = weekday_to_ordinal(date.weekday());
                        if current_days[month_idx] != 1 || ordinal == day % 8 {
                            if date == today {
                                let style = Style::new().reverse();
                                print!("{:>2}", style.paint(current_days[month_idx].to_string()));
                            } else {
                                print!("{:>2}", current_days[month_idx]);
                            }
                            current_days[month_idx] += 1;
                        } else {
                            print!("  ");
                        }
                    } else {
                        print!("  ");
                    }
                    print!(" ");
                }
                print!(" ")
            }
            println!();
        }

        if *month_chunk.last().unwrap() != end {
            println!();
        }
    }

    Ok(())
}
