use std::process;

fn main() {
    if let Err(error) = wcr::run() {
        eprint!("{error}");
        process::exit(1)
    }
}

