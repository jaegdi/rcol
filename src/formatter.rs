use crate::args::AppArgs;
use crate::processor::TableData;
use std::io::{self, Write};
use unicode_width::UnicodeWidthStr;
use regex::Regex;

fn visible_width(s: &str) -> usize {
    // Regex to strip ANSI escape codes
    // CSI: \x1b\[ ... [a-zA-Z]
    // OSC: \x1b\] ... (\x07|\x1b\\)
    let ansi_regex = Regex::new(r"(\x1b\[[0-9;?]*[a-zA-Z])|(\x1b\].*?(\x07|\x1b\\))").unwrap();
    let stripped = ansi_regex.replace_all(s, "");
    UnicodeWidthStr::width(stripped.as_ref())
}

pub fn format_output(data: TableData, args: &AppArgs) -> io::Result<()> {
    if args.csv {
        format_csv(&data, args)
    } else if args.json {
        format_json(&data, args)
    } else if args.html {
        format_html(&data, args)
    } else {
        format_ascii(&data, args)
    }
}

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
                            obj.insert(data.headers[i].clone(), serde_json::Value::String(val.clone()));
                        }
                    }
                    map.insert(key.clone(), serde_json::Value::Object(obj));
                }
            }
            serde_json::to_writer_pretty(&mut handle, &map)?;
        } else {
            let mut arr = Vec::new();
            for row in &data.rows {
                let mut obj = serde_json::Map::new();
                for (i, val) in row.iter().enumerate() {
                    if i < data.headers.len() {
                        obj.insert(data.headers[i].clone(), serde_json::Value::String(val.clone()));
                    }
                }
                arr.push(obj);
            }
            serde_json::to_writer_pretty(&mut handle, &arr)?;
        }
    } else {
        serde_json::to_writer_pretty(&mut handle, &data.rows)?;
    }
    
    writeln!(handle)?;
    Ok(())
}

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
    fn unicode() -> Self {
        Self {
            h: '─', v: '│',
            tl: '┌', tr: '┐',
            bl: '└', br: '┘',
            tm: '┬', bm: '┴',
            lm: '├', rm: '┤',
            c: '┼',
        }
    }
    
    // Fallback if needed, but user requested unicode
    #[allow(dead_code)]
    fn ascii() -> Self {
        Self {
            h: '-', v: '|',
            tl: '+', tr: '+',
            bl: '+', br: '+',
            tm: '+', bm: '+',
            lm: '+', rm: '+',
            c: '+',
        }
    }
}

fn format_ascii(data: &TableData, args: &AppArgs) -> io::Result<()> {
    // Calculate widths
    let mut widths = Vec::new();
    let mut num_cols = 0;
    
    if !data.headers.is_empty() {
        num_cols = data.headers.len();
        for h in &data.headers {
            widths.push(visible_width(h));
        }
    }
    
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

    // Adjust widths for column numbers if -num is set
    if args.num {
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
    
    // Determine padding
    let padding = " ".repeat(args.w);
    let col_sep = &args.colsep; // Used for non-pp mode or internal content separation?
    // If -pp, we use box chars for separation.
    // If NOT -pp, we use col_sep.
    
    let draw_borders = args.pp;
    let draw_ts = args.ts || args.header.is_some();
    let draw_fs = args.fs;
    let draw_cs = args.cs || args.pp;
    
    let chars = BoxChars::unicode();

    let print_separator = |widths: &[usize], left: char, right: char, cross: char, horiz: char| {
        let mut line = String::new();
        if draw_borders { line.push(left); }
        for (i, w) in widths.iter().enumerate() {
            if i > 0 {
                if draw_borders || draw_cs { line.push(cross); }
            }
            let total_w = w + 2 * args.w;
            for _ in 0..total_w {
                line.push(horiz);
            }
        }
        if draw_borders { line.push(right); }
        println!("{}", line);
    };

    // Print Column Numbers if -num is set
    if args.num {
        if draw_borders { 
            // Top border
            print_separator(&widths, chars.tl, chars.tr, chars.tm, chars.h);
        }
        
        let mut line = String::new();
        if draw_borders { line.push(chars.v); }
        for (i, w) in widths.iter().enumerate() {
            if i > 0 {
                if draw_borders { line.push(chars.v); } 
                else if draw_cs { line.push_str(col_sep); }
                else { line.push_str(&padding); }
            }
            let num_str = if i < data.original_column_indices.len() {
                (data.original_column_indices[i] + 1).to_string()
            } else {
                (i + 1).to_string()
            };
            let num_w = visible_width(&num_str);
            line.push_str(&padding);
            line.push_str(&num_str);
            if *w > num_w {
                line.push_str(&" ".repeat(*w - num_w));
            }
            line.push_str(&padding);
        }
        if draw_borders { line.push(chars.v); }
        println!("{}", line);
        
        // Separator between numbers and header/data
        if draw_borders || draw_ts {
             if draw_borders {
                 print_separator(&widths, chars.lm, chars.rm, chars.c, chars.h);
             } else {
                 print_separator(&widths, chars.h, chars.h, chars.h, chars.h);
             }
        }
    } else {
        // No numbers, check if we need top border for header
        if !data.headers.is_empty() && draw_borders {
             print_separator(&widths, chars.tl, chars.tr, chars.tm, chars.h);
        } else if data.headers.is_empty() && draw_borders {
             // No header, just data top border
             print_separator(&widths, chars.tl, chars.tr, chars.tm, chars.h);
        }
    }

    // Print Header
    if !data.headers.is_empty() {
        let mut line = String::new();
        if draw_borders { line.push(chars.v); }
        
        for (i, h) in data.headers.iter().enumerate() {
            if i > 0 {
                if draw_borders { line.push(chars.v); } 
                else if draw_cs { line.push_str(col_sep); }
                else { line.push_str(&padding); }
            } else if !draw_borders {
                // No leading separator unless borders
            }

            let align_right = h.starts_with('-');
            let content = if align_right { &h[1..] } else { h };
            let content_w = visible_width(content);
            
            let w = widths[i];
            if args.nf {
                line.push_str(content);
            } else {
                line.push_str(&padding);
                let pad_len = w.saturating_sub(content_w);
                let pad = " ".repeat(pad_len);
                if align_right {
                    line.push_str(&pad);
                    line.push_str(content);
                } else {
                    line.push_str(content);
                    line.push_str(&pad);
                }
                line.push_str(&padding);
            }
        }
        if draw_borders { line.push(chars.v); }
        println!("{}", line);
        
        if draw_ts {
            if draw_borders {
                print_separator(&widths, chars.lm, chars.rm, chars.c, chars.h);
            } else {
                 print_separator(&widths, chars.h, chars.h, chars.h, chars.h);
            }
        }
    }

    // Print Rows
    for (row_idx, row) in data.rows.iter().enumerate() {
        // Footer Separator: before the last row
        if draw_fs && row_idx > 0 && row_idx == data.rows.len() - 1 {
             if draw_borders {
                 print_separator(&widths, chars.lm, chars.rm, chars.c, chars.h);
             } else {
                 print_separator(&widths, chars.h, chars.h, chars.h, chars.h);
             }
        }

        let mut line = String::new();
        if draw_borders { line.push(chars.v); }
        
        for (i, val) in row.iter().enumerate() {
            if i > 0 {
                if draw_borders { line.push(chars.v); } 
                else if draw_cs { line.push_str(col_sep); }
                else { line.push_str(&padding); }
            }
            
            let w = if i < widths.len() { widths[i] } else { visible_width(val) };
            
            if args.nf {
                line.push_str(val);
            } else {
                line.push_str(&padding);
                let is_num = !args.nn && val.parse::<f64>().is_ok();
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
                line.push_str(&padding);
            }
        }
        if draw_borders { line.push(chars.v); }
        println!("{}", line);
    }
    
    // Bottom Border
    if draw_borders {
        print_separator(&widths, chars.bl, chars.br, chars.bm, chars.h);
    }

    Ok(())
}
