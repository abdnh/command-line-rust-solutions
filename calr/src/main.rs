use std::process;

fn main() {
    if let Err(error) = calr::run() {
        eprintln!("{error}");
        process::exit(1)
    }
}
