// Filesystem manipulation
use rev_buf_reader::RevBufReader;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
// Error Handling
use log::debug;
use miette::{Error, IntoDiagnostic, Result};

/**
* Read the last line of the file at the provide path
* and return it as a string.
*/
pub fn read_last_line(path: &Path) -> Result<String> {
    let file = File::open(path).into_diagnostic()?;
    let buf = RevBufReader::new(file);
    let mut lines = buf.lines();
    let last_line = lines.next();
    if let Some(last_line) = last_line {
        Ok(last_line.into_diagnostic()?)
    } else {
        let message = format!("Empty file: {}", path.display());
        debug!("{}", message);
        Err(Error::msg(message))
    }
}
