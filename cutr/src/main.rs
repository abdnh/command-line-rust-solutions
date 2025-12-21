use std::process;

fn main() {
    if let Err(error) = cutr::run() {
        eprint!("{error}");
        process::exit(1)
    }
}
