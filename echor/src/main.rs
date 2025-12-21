use clap::builder::{Arg, ArgAction, Command};

fn main() {
    let matches = Command::new("echor")
        .version("0.1.0")
        .author("Abdo")
        .about("Rust echo")
        .arg(
            Arg::new("text")
                .value_name("TEXT")
                .help("Input text")
                .required(true).action(ArgAction::Append),
        )
        .arg(
            Arg::new("omit_newline")
                .short('n')
                .help("Do not print newline")
                .action(ArgAction::SetTrue),
        )
        .get_matches();
    let text: String = matches.get_many::<String>("text").unwrap_or_default().map(|s| s.as_str()).collect::<Vec<_>>().join(" ");
    let ending = if matches.get_flag("omit_newline") {
        ""
    } else {
        "\n"
    };
    print!("{}{}", text, ending);
}
