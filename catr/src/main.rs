use catr::CliError;


fn main() {
    if let Err(error) = catr::run() {
        eprint!("{}", error);
        let mut ret = 1;
        if matches!(error, CliError::Io {..}) {
            ret = 0;
        }
        std::process::exit(ret);
    }
}
