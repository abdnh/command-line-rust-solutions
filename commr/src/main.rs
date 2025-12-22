use std::process;

fn main() {
    if let Err(error) = commr::run() {
        eprintln!("{error}");
        process::exit(1)
    }
}
