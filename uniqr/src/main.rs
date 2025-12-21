use std::process;

fn main() {
    if let Err(error) = uniqr::run() {
        eprint!("{error}");
        process::exit(1)
    }
}
