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

/// Print comprehensive man page for rcol
fn print_manpage() {
    let version = env!("CARGO_PKG_VERSION");
    let author = env!("CARGO_PKG_AUTHORS");

    let mantext = r##"
    rcol(1)                          General Commands Manual                         rcol(1)

    NAME
           rcol - Rust Column Formatter: Format and shape unformatted ASCII text into columns

    SYNOPSIS
           rcol [OPTIONS] [COLUMNS...]

    DESCRIPTION
           rcol formats unformatted ASCII text columns into neatly aligned columns. It can read input
           from standard input or a specified file, process the data (sorting, grouping,
           filtering), and output in various formats including plain text, CSV, JSON, or HTML.

    OPTIONS
           -f, --file FILENAME           Read input from FILENAME instead of standard input
           -H, --header LINE            Define a custom header line for the output
           -s, --sep SEPARATOR          Define the input separator (default: whitespace)
           -m, --mb                     Treat multiple consecutive separators as a single delimiter
           -w, --width WIDTH            Set padding width between columns (default: 1)
           -C, --colsep SEPARATOR       Define column separation string (default: '│')
           -F, --filter REGEX           Process only lines matching the given regular expression
           -S, --sortcol N              Sort output by column N (1-based index)
           -g, --gcol N                 Group output by column N
           -gcolval                     Keep repeated group values instead of replacing with empty strings
           --nf                         No Format: Do not align columns to a common width
           --nn                         No Numerical: Disable automatic right-alignment of numerical values
           --nhl                        No Headline: Treat first line as data, not a header
           --ts                         Title Separator: Draw line between header and data
           --fs                         Footer Separator: Draw line before last row of data
           --cs                         Column Separator: Draw vertical line between columns
           -p, --pp                     Pretty Print: Draw border around table with Unicode box characters
           --rh                         Remove Header: Discard first line of input
           -n, --num                    Numbering: Add row with column numbers at top
           --csv                        Output as CSV format
           --json                       Output as JSON format
           --yaml                       Output as YAML format
           --html                       Output as HTML format
           --jtc                        JSON Title Column: Use first column as key for JSON objects
           -v, --verify                 Print parameter verification info
           -M, --manpage                Output comprehensive man page
           COLUMNS                      Specify which columns to output (1-based indices)

    EXAMPLES
           # Format input from stdin with default settings
           cat data.txt | rcol

           # Format specific columns with custom separator and pretty print
           rcol -s ',' -p 1 3 5 < data.csv

           # Group by column 2 and sort by column 1
           rcol -g 2 -S 1 data.txt

           # Convert to JSON with first column as keys
           rcol --json --jtc data.txt

    SEE ALSO
           column(1), fmt(1)

    VERSION
           {version}

    AUTHOR
           {author}
    "##;
    println!("{}", mantext
        .replace("{version}", version)
        .replace("{author}", author)
    );
}

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

    if args.manpage {
        print_manpage();
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