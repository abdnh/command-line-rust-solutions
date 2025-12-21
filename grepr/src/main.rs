use std::process;

fn main() {
    if let Err(error) = grepr::run() {
        eprintln!("{error}");
        process::exit(1)
    }
}
