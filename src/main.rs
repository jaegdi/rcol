mod args;
mod formatter;
mod input;
mod processor;

use args::AppArgs;
use clap::Parser;
use formatter::format_output;
use input::read_input;
use processor::process_input;
use std::process;

/// Main entry point for the rcol application.
///
/// Parses command-line arguments, reads input from file or stdin, processes the data
/// according to the specified options, and formats the output in the requested format.
/// Exits with status code 1 on any error.
fn main() {
    let args = AppArgs::parse();

    if args.verify {
        println!("Args: {:?}", args);
        return;
    }

    // Read input
    let lines = match read_input(&args) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            process::exit(1);
        }
    };

    // Process input
    let processed_data = match process_input(lines, &args) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error processing input: {}", e);
            process::exit(1);
        }
    };

    // Format output
    if let Err(e) = format_output(processed_data, &args) {
        eprintln!("Error formatting output: {}", e);
        process::exit(1);
    }
}
