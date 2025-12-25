use std::process;

fn main() {
    if let Err(error) = fortuner::run() {
        eprintln!("{error}");
        process::exit(1)
    }
}
