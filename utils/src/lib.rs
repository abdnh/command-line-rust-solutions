use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

/// Like [`std::io::Lines`] but preserves line endings
pub struct EolPreservingLines<B> {
    buf: B,
}

impl<B: BufRead> Iterator for EolPreservingLines<B> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<io::Result<String>> {
        let mut buf = String::new();
        match self.buf.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_n) => Some(Ok(buf)),
            Err(e) => Some(Err(e)),
        }
    }
}

pub trait LinesWithEol<T> {
    fn lines_with_eol(self) -> EolPreservingLines<T>;
}

impl<T: BufRead> LinesWithEol<T> for T {
    fn lines_with_eol(self) -> EolPreservingLines<T> {
        EolPreservingLines { buf: self }
    }
}

pub fn reader_from_path<T: AsRef<Path>>(path: T) -> std::io::Result<impl BufRead> {
    let is_stdin = path.as_ref().to_str().map(|s| s == "-").unwrap_or(false);
    let buffer: Box<dyn BufRead> = if is_stdin {
        Box::new(std::io::stdin().lock())
    } else {
        Box::new(BufReader::new(File::open(path)?))
    };

    Ok(buffer)
}
