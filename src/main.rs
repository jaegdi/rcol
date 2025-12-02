mod args;
mod input;
mod processor;
mod formatter;

use args::parse_args;
use input::read_input;
use processor::process_input;
use formatter::format_output;
use std::process;

fn main() {
    match parse_args() {
        Ok(args) => {
            if args.help {
                print_help();
                return;
            }
            if args.man {
                print_man();
                return;
            }
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
            let table_data = match process_input(lines, &args) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error processing input: {}", e);
                    process::exit(1);
                }
            };

            // Format output
            if let Err(e) = format_output(table_data, &args) {
                eprintln!("Error formatting output: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("Usage: rcol [options] [columns]");
    println!("Options:");
    println!("  -file=filename      Read input from file");
    println!("  -header='...'       Define header line");
    println!("  -sep=' '            Separator (default ' ')");
    println!("  -mb                 More blanks");
    println!("  -w=1                Blanks between columns (default 1)");
    println!("  -colsep='|'         Output column separator (default '|')");
    println!("  -filter='regex'     Filter lines");
    println!("  -sortcol=n          Sort by column n");
    println!("  -gcol=n             Group by column n");
    println!("  -gcolval            Keep group column values");
    println!("  -nf                 No format");
    println!("  -nn                 No numerical alignment");
    println!("  -nhl                No headline");
    println!("  -ts                 Title separator");
    println!("  -fs                 Footer separator");
    println!("  -cs                 Column separator");
    println!("  -pp                 Pretty print");
    println!("  -rh                 Remove header");
    println!("  -num                Numbering");
    println!("  -csv                CSV output");
    println!("  -json               JSON output");
    println!("  -html               HTML output");
    println!("  -jtc                Title column for JSON");
    println!("  -help, -h           Show help");
    println!("  -man                Show manual");
    println!("  -v, -verify         Verify parameters");
}

fn print_man() {
    print_help();
    println!("\nManual:");
    println!("  rcol formats text columns from stdin or file.");
    println!("  It supports filtering, sorting, grouping, and various output formats.");
}
