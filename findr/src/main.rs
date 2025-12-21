use std::process;

fn main() {
    if let Err(error) = findr::run() {
        eprint!("{error}");
        process::exit(1)
    }
}
