use crate::args::AppArgs;
use std::fs::File;
use std::io::{self, BufRead, BufReader, IsTerminal};

/// Reads input lines from a file and/or stdin, joining lines with unclosed quotes.
///
/// If a file is specified via `args.file`, reads all lines from that file.
/// Additionally reads from stdin if it's not a terminal (piped input) or if no file
/// was specified. This allows combining file and piped input when both are provided.
///
/// Lines with unclosed double quotes (") or single quotes (') are joined with the
/// next line, allowing quoted strings to span multiple lines.
///
/// # Arguments
///
/// * `args` - Application arguments containing the optional file path
///
/// # Returns
///
/// - `Ok(Vec<String>)` containing all input lines (with multi-line quotes joined)
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
    let stdin = io::stdin();
    if !stdin.is_terminal() || args.file.is_none() {
        let reader = stdin.lock();
        for line in reader.lines() {
           lines.push(line?.trim().to_string());
        }
    }

    // Join lines with unclosed quotes
    let joined_lines = join_quoted_lines(lines);
    Ok(joined_lines)
}

/// Joins lines that have unclosed quotes with the following line.
///
/// A line is considered to have unclosed quotes if the count of unescaped
/// double quotes or single quotes is odd.
///
/// # Arguments
///
/// * `lines` - Raw input lines (possibly with unclosed quotes)
///
/// # Returns
///
/// A vector of lines where multi-line quoted strings are joined
fn join_quoted_lines(lines: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_line = String::new();
    
    for line in lines {
        if current_line.is_empty() {
            current_line = line;
        } else {
            current_line.push('\n');
            current_line.push_str(&line);
        }
        
        // Check if this line has balanced quotes
        if has_balanced_quotes(&current_line) {
            result.push(current_line.clone());
            current_line = String::new();
        }
    }
    
    // Add any remaining line
    if !current_line.is_empty() {
        result.push(current_line);
    }
    
    result
}

/// Checks if a line has balanced quotes (all quotes are closed).
///
/// Counts unescaped double quotes and single quotes separately.
/// A line has balanced quotes if both counts are even.
///
/// # Arguments
///
/// * `line` - The line to check
///
/// # Returns
///
/// `true` if all quotes are balanced (closed), `false` if unclosed
fn has_balanced_quotes(line: &str) -> bool {
    let mut double_quote_count = 0;
    let mut single_quote_count = 0;
    let mut escape_next = false;
    
    for ch in line.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }
        
        match ch {
            '\\' => escape_next = true,
            '"' => double_quote_count += 1,
            '\'' => single_quote_count += 1,
            _ => {}
        }
    }
    
    // Balanced if both counts are even
    double_quote_count % 2 == 0 && single_quote_count % 2 == 0
}

