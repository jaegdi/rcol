use crate::args::AppArgs;
use std::fs::File;
use std::io::{self, BufRead, BufReader, IsTerminal};

/// Reads input lines from a file and/or stdin based on application arguments.
///
/// If a file is specified via `args.file`, reads all lines from that file.
/// Additionally reads from stdin if it's not a terminal (piped input) or if no file
/// was specified. This allows combining file and piped input when both are provided.
///
/// # Arguments
///
/// * `args` - Application arguments containing the optional file path
///
/// # Returns
///
/// - `Ok(Vec<String>)` containing all input lines
/// - `Err(io::Error)` if file reading or stdin reading fails
///
/// # Examples
///
/// - File only: `rcol -file=data.txt`
/// - Stdin only: `cat data.txt | rcol`
/// - Both: `cat extra.txt | rcol -file=data.txt` (combines both sources)
pub fn read_input(args: &AppArgs) -> io::Result<Vec<String>> {
    let mut lines = Vec::new();

    // Read from file if specified
    if let Some(filename) = &args.file {
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
           lines.push(line?.trim().to_string());
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
           lines.push(line?.trim().to_string());
        }
    }

    Ok(lines)
}
