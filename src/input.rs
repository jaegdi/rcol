use std::fs::File;
use std::io::{self, BufRead, BufReader, IsTerminal};
use crate::args::AppArgs;

pub fn read_input(args: &AppArgs) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();

    // Read from file if specified
    if let Some(filename) = &args.file {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            lines.push(line?);
        }
    }

    // Read from stdin if it's not a terminal (piped input) or if no file was specified
    // If no file specified and it IS a terminal, we still read (interactive mode like cat)
    // But if file IS specified and stdin IS a terminal, we probably skip stdin to avoid hanging?
    // Requirement: "if there is also data from STDIN, this is added together"
    // This usually implies piped data.
    
    let stdin = io::stdin();
    if !stdin.is_terminal() || args.file.is_none() {
        let reader = stdin.lock();
        for line in reader.lines() {
            lines.push(line?);
        }
    }

    Ok(lines)
}
