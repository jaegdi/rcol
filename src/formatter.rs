use crate::args::AppArgs;
use crate::processor::TableData;
use regex::Regex;
use serde_yaml::{Mapping, Value};
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;

/// Calculates the visible width of a string, accounting for Unicode and ANSI escape codes.
///
/// Strips ANSI escape sequences (CSI and OSC codes) before calculating the display width
/// using Unicode width rules. This ensures proper alignment in terminal output.
///
/// # Arguments
///
/// * `s` - The string to measure
///
/// # Returns
///
/// The visible width in character cells (not bytes)
/// Strips ANSI escape sequences from a string.
///
/// # Arguments
///
/// * `s` - The string to strip
///
/// # Returns
///
/// A new String with ANSI codes removed
fn strip_ansi(s: &str) -> String {
    // Regex to strip ANSI escape codes
    // CSI: \x1b\[ ... [a-zA-Z]
    // OSC: \x1b\] ... (\x07|\x1b\\)
    let ansi_regex = Regex::new(r"(\x1b\[[0-9;?]*[a-zA-Z])|(\x1b\].*?(\x07|\x1b\\))").unwrap();
    ansi_regex.replace_all(s, "").to_string()
}

/// Calculates the visible width of a string, accounting for Unicode and ANSI escape codes.
///
/// Strips ANSI escape sequences (CSI and OSC codes) before calculating the display width
/// using Unicode width rules. This ensures proper alignment in terminal output.
///
/// # Arguments
///
/// * `s` - The string to measure
///
/// # Returns
///
/// The visible width in character cells (not bytes)
fn visible_width(s: &str) -> usize {
    let stripped = strip_ansi(s);
    UnicodeWidthStr::width(stripped.as_str())
}

/// Formats and outputs table data according to the specified format.
///
/// Routes to the appropriate formatter based on output format flags:
/// - CSV (`-csv`)
/// - JSON (`-json`)
/// - HTML (`-html`)
/// - ASCII table (default)
///
/// # Arguments
///
/// * `data` - Processed table data to format
/// * `args` - Application arguments specifying output format and options
///
/// # Returns
///
/// - `Ok(())` if output succeeds
/// - `Err(io::Error)` if writing to stdout fails
pub fn format_output(data: TableData, args: &AppArgs) -> io::Result<()> {
    if args.csv {
        format_csv(&data, args)
    } else if args.json {
        format_json(&data, args)
    } else if args.yaml {
        format_yaml(&data, args)
    } else if args.html {
        format_html(&data, args)
    } else {
        format_ascii(&data, args)
    }
}

/// Formats table data as CSV output.
///
/// Outputs headers (if present) followed by all data rows in standard CSV format,
/// with proper escaping and quoting as needed.
///
/// # Arguments
///
/// * `data` - Table data to format
/// * `_args` - Application arguments (currently unused for CSV formatting)
///
/// # Returns
///
/// - `Ok(())` if output succeeds
/// - `Err(io::Error)` if writing fails
fn format_csv(data: &TableData, _args: &AppArgs) -> io::Result<()> {
    let mut wtr = csv::Writer::from_writer(io::stdout());

    if !data.headers.is_empty() {
        wtr.write_record(&data.headers)?;
    }

    for row in &data.rows {
        wtr.write_record(row)?;
    }

    wtr.flush()?;
    Ok(())
}

/// Formats table data as YAML output.
///
/// Supports two modes:
/// - Standard: List of maps (rows), each row is a map with header keys
/// - Title column mode (`-jtc`): Object keyed by first column, nested objects for remaining columns
///
/// # Arguments
///
/// * `data` - Table data to format
/// * `args` - Application arguments (checks `-jtc` flag)
///
/// # Returns
///
/// - `Ok(())` if output succeeds
/// - `Err(io::Error)` if writing fails
fn format_yaml(data: &TableData, args: &AppArgs) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if !data.headers.is_empty() {
        if args.jtc {
            let mut map = Mapping::new();
            for row in &data.rows {
                if let Some(key) = row.first() {
                    let mut obj = Mapping::new();
                    for (i, val) in row.iter().enumerate().skip(1) {
                        if i < data.headers.len() {
                            obj.insert(
                                Value::String(strip_ansi(&data.headers[i])),
                                Value::String(strip_ansi(val)),
                            );
                        }
                    }
                    map.insert(Value::String(strip_ansi(key)), Value::Mapping(obj));
                }
            }
            write!(
                handle,
                "{}",
                serde_yaml::to_string(&map).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            )?;
        } else {
            let mut arr = Vec::new();
            for row in &data.rows {
                let mut obj = Mapping::new();
                for (i, val) in row.iter().enumerate() {
                    if i < data.headers.len() {
                        obj.insert(
                            Value::String(strip_ansi(&data.headers[i])),
                            Value::String(strip_ansi(val)),
                        );
                    }
                }
                arr.push(Value::Mapping(obj));
            }
            write!(
                handle,
                "{}",
                serde_yaml::to_string(&arr).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
            )?;
        }
    } else {
        // Strip ANSI from raw rows if no headers
        let stripped_rows: Vec<Vec<String>> = data
            .rows
            .iter()
            .map(|row| row.iter().map(|s| strip_ansi(s)).collect())
            .collect();
        write!(
            handle,
            "{}",
            serde_yaml::to_string(&stripped_rows)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        )?;
    }

    writeln!(handle)?;
    Ok(())
}

/// Formats table data as JSON output.
///
/// Supports two output modes:
/// - Standard: Array of objects, where each object represents a row with header keys
/// - Title column mode (`-jtc`): Object keyed by first column, with nested objects for remaining columns
///
/// # Arguments
///
/// * `data` - Table data to format
/// * `args` - Application arguments (checks `-jtc` flag)
///
/// # Returns
///
/// - `Ok(())` if output succeeds
/// - `Err(io::Error)` if writing fails
fn format_json(data: &TableData, args: &AppArgs) -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if !data.headers.is_empty() {
        if args.jtc {
            let mut map = serde_json::Map::new();
            for row in &data.rows {
                if let Some(key) = row.first() {
                    let mut obj = serde_json::Map::new();
                    for (i, val) in row.iter().enumerate().skip(1) {
                        if i < data.headers.len() {
                            obj.insert(
                                strip_ansi(&data.headers[i]),
                                serde_json::Value::String(strip_ansi(val)),
                            );
                        }
                    }
                    map.insert(strip_ansi(key), serde_json::Value::Object(obj));
                }
            }
            serde_json::to_writer_pretty(&mut handle, &map)?;
        } else {
            let mut arr = Vec::new();
            for row in &data.rows {
                let mut obj = serde_json::Map::new();
                for (i, val) in row.iter().enumerate() {
                    if i < data.headers.len() {
                        obj.insert(
                            strip_ansi(&data.headers[i]),
                            serde_json::Value::String(strip_ansi(val)),
                        );
                    }
                }
                arr.push(obj);
            }
            serde_json::to_writer_pretty(&mut handle, &arr)?;
        }
    } else {
        // Strip ANSI from raw rows if no headers
        let stripped_rows: Vec<Vec<String>> = data
            .rows
            .iter()
            .map(|row| row.iter().map(|s| strip_ansi(s)).collect())
            .collect();
        serde_json::to_writer_pretty(&mut handle, &stripped_rows)?;
    }

    writeln!(handle)?;
    Ok(())
}

/// Formats table data as HTML table output.
///
/// Generates a complete HTML table with proper thead/tbody structure.
/// Headers are output in `<th>` tags, data rows in `<td>` tags.
///
/// # Arguments
///
/// * `data` - Table data to format
/// * `_args` - Application arguments (currently unused for HTML formatting)
///
/// # Returns
///
/// - `Ok(())` if output succeeds
/// - `Err(io::Error)` if writing fails
fn format_html(data: &TableData, _args: &AppArgs) -> io::Result<()> {
    println!("<table>");
    if !data.headers.is_empty() {
        println!("  <thead>");
        println!("    <tr>");
        for h in &data.headers {
            println!("      <th>{}</th>", h);
        }
        println!("    </tr>");
        println!("  </thead>");
    }
    println!("  <tbody>");
    for row in &data.rows {
        println!("    <tr>");
        for val in row {
            println!("      <td>{}</td>", val);
        }
        println!("    </tr>");
    }
    println!("  </tbody>");
    println!("</table>");
    Ok(())
}

/// Unicode box-drawing characters for table formatting.
///
/// Contains all the characters needed to draw table borders and separators
/// using Unicode box-drawing characters or ASCII fallbacks.
struct BoxChars {
    h: char,
    v: char,
    tl: char,
    tr: char,
    bl: char,
    br: char,
    tm: char,
    bm: char,
    lm: char,
    rm: char,
    c: char,
}

impl BoxChars {
    /// Creates a `BoxChars` instance with Unicode box-drawing characters.
    ///
    /// Uses proper Unicode characters for smooth, professional-looking tables.
    fn unicode() -> Self {
        Self {
            h: '─',
            v: '│',
            tl: '┌',
            tr: '┐',
            bl: '└',
            br: '┘',
            tm: '┬',
            bm: '┴',
            lm: '├',
            rm: '┤',
            c: '┼',
        }
    }

    /// Creates a `BoxChars` instance with ASCII characters.
    ///
    /// Fallback option using basic ASCII characters (+, -, |) for environments
    /// that don't support Unicode box-drawing characters.
    // Fallback if needed, but user requested unicode
    #[allow(dead_code)]
    fn ascii() -> Self {
        Self {
            h: '-',
            v: '|',
            tl: '+',
            tr: '+',
            bl: '+',
            br: '+',
            tm: '+',
            bm: '+',
            lm: '+',
            rm: '+',
            c: '+',
        }
    }
}

/// Formats table data as an ASCII/Unicode table with borders and alignment.
///
/// The primary formatting function that handles:
/// - Column width calculation based on content
/// - Proper alignment (numeric values right-aligned, text left-aligned)
/// - Optional features: borders (`-pp`), separators (`-ts`, `-fs`, `-cs`), numbering (`-num`)
/// - Padding and spacing control (`-w`)
///
/// # Arguments
///
/// * `data` - Table data to format
/// * `args` - Application arguments controlling formatting options
///
/// # Returns
///
/// - `Ok(())` if output succeeds
/// - `Err(io::Error)` if writing fails
///
/// # Formatting Behavior
///
/// - Automatically calculates column widths based on content
/// - Right-aligns numeric values unless `-nn` is set
/// - Left-aligns text values
/// - Headers starting with '-' are right-aligned
/// - Draws Unicode box characters for pretty printing when `-pp` is enabled
/// Context for rendering the table.
struct RenderContext<'a> {
    widths: &'a [usize],
    args: &'a AppArgs,
    chars: BoxChars,
    col_sep: &'a str,
    padding: String,
    draw_borders: bool,
    draw_cs: bool,
    draw_ts: bool,
    draw_fs: bool,
}

/// Formats table data as an ASCII/Unicode table with borders and alignment.
fn format_ascii(data: &TableData, args: &AppArgs) -> io::Result<()> {
    let widths = calculate_widths(data, args);
    let padding = " ".repeat(args.w);
    let col_sep = &args.colsep;
    let chars = BoxChars::unicode();

    let draw_borders = args.pp;
    let draw_ts = args.ts || args.header.is_some();
    let draw_fs = args.fs;
    let draw_cs = args.cs || args.pp;

    let ctx = RenderContext {
        widths: &widths,
        args,
        chars,
        col_sep,
        padding,
        draw_borders,
        draw_cs,
        draw_ts,
        draw_fs,
    };

    // Print Column Numbers
    if args.num {
        print_column_numbers(data, &ctx);
    } else {
        // No numbers, check if we need top border for header or data
        if draw_borders {
            print_separator(&ctx, ctx.chars.tl, ctx.chars.tr, ctx.chars.tm, ctx.chars.h);
        }
    }

    // Print Header
    if !data.headers.is_empty() {
        print_header(data, &ctx);
    }

    // Print Rows
    print_data_rows(data, &ctx);

    // Bottom Border
    if draw_borders {
        print_separator(&ctx, ctx.chars.bl, ctx.chars.br, ctx.chars.bm, ctx.chars.h);
    }

    Ok(())
}

/// Calculates the width of each column based on data content and headers.
///
/// Also handles adjusting widths for the column numbering row if `-num` is specified.
///
/// # Arguments
///
/// * `data` - The table data containing headers and rows
/// * `args` - Application arguments
///
/// # Returns
///
/// A vector of column widths
fn calculate_widths(data: &TableData, args: &AppArgs) -> Vec<usize> {
    let mut widths = Vec::new();
    let mut num_cols = 0;

    if !data.headers.is_empty() {
        num_cols = data.headers.len();
        // Initial width based on headers
        for h in &data.headers {
            widths.push(visible_width(h));
        }
    }

    // Update widths based on max content length in rows
    for row in &data.rows {
        if row.len() > num_cols {
            num_cols = row.len();
            widths.resize(num_cols, 0);
        }
        for (i, val) in row.iter().enumerate() {
            if i < widths.len() {
                let w = visible_width(val);
                if w > widths[i] {
                    widths[i] = w;
                }
            }
        }
    }

    if args.num {
        // Adjust for column numbers if needed
        for i in 0..widths.len() {
            let num_str = if i < data.original_column_indices.len() {
                (data.original_column_indices[i] + 1).to_string()
            } else {
                (i + 1).to_string()
            };
            let num_w = visible_width(&num_str);
            if num_w > widths[i] {
                widths[i] = num_w;
            }
        }
    }
    widths
}

/// Prints a horizontal separator line.
///
/// Handles different styles for borders (`-pp`) and standard separators.
///
/// # Arguments
///
/// * `ctx` - Render context
/// * `left` - Character for the left edge
/// * `right` - Character for the right edge
/// * `cross` - Character for column intersections
/// * `horiz` - Character for the horizontal line
fn print_separator(ctx: &RenderContext, left: char, right: char, cross: char, horiz: char) {
    let mut line = String::new();

    if ctx.draw_borders {
        line.push(left);
    }
    for (i, w) in ctx.widths.iter().enumerate() {
        if i > 0 {
            if ctx.draw_borders || ctx.draw_cs {
                line.push(cross);
            } else {
                // Fill space between columns with horizontal line if no vertical separator
                for _ in 0..ctx.args.w {
                    line.push(horiz);
                }
            }
        }
        let total_w = w + 2 * ctx.args.w;
        for _ in 0..total_w {
            line.push(horiz);
        }
    }
    if ctx.draw_borders {
        line.push(right);
    }
    println!("{}", line);
}

/// Prints the row containing column numbers.
///
/// Used when the `-num` flag is active. Handles formatting and alignment
/// of column indices.
///
/// # Arguments
///
/// * `data` - Table data
/// * `ctx` - Render context
fn print_column_numbers(data: &TableData, ctx: &RenderContext) {
    if ctx.draw_borders {
        print_separator(ctx, ctx.chars.tl, ctx.chars.tr, ctx.chars.tm, ctx.chars.h);
    }

    let mut line = String::new();
    if ctx.draw_borders {
        line.push(ctx.chars.v);
    }
    for (i, w) in ctx.widths.iter().enumerate() {
        if i > 0 {
            if ctx.draw_borders {
                line.push(ctx.chars.v);
            } else if ctx.draw_cs {
                line.push_str(ctx.col_sep);
            } else {
                line.push_str(&ctx.padding);
            }
        }
        let num_str = if i < data.original_column_indices.len() {
            (data.original_column_indices[i] + 1).to_string()
        } else {
            (i + 1).to_string()
        };
        // Calculate width for alignment
        let num_w = visible_width(&num_str);
        line.push_str(&ctx.padding);
        line.push_str(&num_str);
        if *w > num_w {
            line.push_str(&" ".repeat(*w - num_w));
        }
        line.push_str(&ctx.padding);
    }
    if ctx.draw_borders {
        line.push(ctx.chars.v);
    }
    println!("{}", line);

    if ctx.draw_borders || ctx.draw_ts {
        if ctx.draw_borders {
            print_separator(ctx, ctx.chars.lm, ctx.chars.rm, ctx.chars.c, ctx.chars.h);
        } else {
            print_separator(ctx, ctx.chars.h, ctx.chars.h, ctx.chars.h, ctx.chars.h);
        }
    }
}

/// Prints the header row.
///
/// Handles alignment of header text (right-aligned if starting with `-`).
///
/// # Arguments
///
/// * `data` - Table data
/// * `ctx` - Render context
fn print_header(data: &TableData, ctx: &RenderContext) {
    let mut line = String::new();
    if ctx.draw_borders {
        line.push(ctx.chars.v);
    }

    for (i, h) in data.headers.iter().enumerate() {
        if i > 0 {
            if ctx.draw_borders {
                line.push(ctx.chars.v);
            } else if ctx.draw_cs {
                line.push_str(ctx.col_sep);
            } else {
                line.push_str(&ctx.padding);
            }
        }

        // Check for right alignment marker
        let align_right = h.starts_with('-');
        let content = if align_right { &h[1..] } else { h };
        let content_w = visible_width(content);

        let w = ctx.widths[i];
        if ctx.args.nf {
            line.push_str(content);
        } else {
            // Apply padding for alignment
            line.push_str(&ctx.padding);
            let pad_len = w.saturating_sub(content_w);
            let pad = " ".repeat(pad_len);
            if align_right {
                line.push_str(&pad);
                line.push_str(content);
            } else {
                line.push_str(content);
                line.push_str(&pad);
            }
            line.push_str(&ctx.padding);
        }
    }
    if ctx.draw_borders {
        line.push(ctx.chars.v);
    }
    println!("{}", line);

    if ctx.draw_ts {
        if ctx.draw_borders {
            print_separator(ctx, ctx.chars.lm, ctx.chars.rm, ctx.chars.c, ctx.chars.h);
        } else {
            print_separator(ctx, ctx.chars.h, ctx.chars.h, ctx.chars.h, ctx.chars.h);
        }
    }
}

/// Prints the data rows.
///
/// Handles formatting of individual cells, including alignment (numeric vs text)
/// and padding. Also handles the footer separator if enabled.
///
/// # Arguments
///
/// * `data` - Table data
/// * `ctx` - Render context
fn print_data_rows(data: &TableData, ctx: &RenderContext) {
    for (row_idx, row) in data.rows.iter().enumerate() {
        if ctx.draw_fs && row_idx > 0 && row_idx == data.rows.len() - 1 {
            if ctx.draw_borders {
                print_separator(ctx, ctx.chars.lm, ctx.chars.rm, ctx.chars.c, ctx.chars.h);
            } else {
                print_separator(ctx, ctx.chars.h, ctx.chars.h, ctx.chars.h, ctx.chars.h);
            }
        }

        let mut line = String::new();
        if ctx.draw_borders {
            line.push(ctx.chars.v);
        }

        for (i, val) in row.iter().enumerate() {
            if i > 0 {
                if ctx.draw_borders {
                    line.push(ctx.chars.v);
                } else if ctx.draw_cs {
                    line.push_str(ctx.col_sep);
                } else {
                    line.push_str(&ctx.padding);
                }
            }

            let w = if i < ctx.widths.len() {
                ctx.widths[i]
            } else {
                visible_width(val)
            };

            if ctx.args.nf {
                line.push_str(val);
            } else {
                line.push_str(&ctx.padding);
                // Check if value is numeric for default right-alignment
                let is_num = !ctx.args.nn && val.parse::<f64>().is_ok();
                let val_w = visible_width(val);
                let pad_len = w.saturating_sub(val_w);
                let pad = " ".repeat(pad_len);

                if is_num {
                    line.push_str(&pad);
                    line.push_str(val);
                } else {
                    line.push_str(val);
                    line.push_str(&pad);
                }
                line.push_str(&ctx.padding);
            }
        }
        if ctx.draw_borders {
            line.push(ctx.chars.v);
        }
        println!("{}", line);
    }
}
