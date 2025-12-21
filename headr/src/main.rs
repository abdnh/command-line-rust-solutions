use std::process;

fn main() {
    if let Err(error) = headr::run() {
        eprint!("{}", error);
        process::exit(1)
    }
}

