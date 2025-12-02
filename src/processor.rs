use crate::args::AppArgs;
use regex::Regex;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub original_column_indices: Vec<usize>,
}

pub fn process_input(lines: Vec<String>, args: &AppArgs) -> Result<TableData, String> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut headers: Vec<String> = Vec::new();

    // 1. Filter lines
    let filter_regex = if let Some(pattern) = &args.filter {
        Some(Regex::new(pattern).map_err(|e| format!("Invalid filter regex: {}", e))?)
    } else {
        None
    };

    let mut filtered_lines = Vec::new();
    for line in lines {
        if let Some(re) = &filter_regex {
            if !re.is_match(&line) {
                continue;
            }
        }
        filtered_lines.push(line);
    }

    if filtered_lines.is_empty() {
        return Ok(TableData { headers, rows, original_column_indices: Vec::new() });
    }

    // 2. Split lines into columns
    // Determine separator regex
    let sep_regex = if args.mb {
        Regex::new(r"\s+").unwrap() // More blanks -> split by one or more whitespace
    } else {
        // Escape the separator if it's a special regex character
        let sep_pattern = regex::escape(&args.sep);
        Regex::new(&sep_pattern).unwrap()
    };

    // Handle Header
    // If -header is provided, use it.
    // If -nhl (no headline) is NOT set, and no -header provided, assume first line is header?
    // Requirement: "-header='...' Headerline, if the text has no headers, you can define headers."
    // "-nhl no headline The data contains no headline."
    // This implies:
    // If -header is set: Use it as header.
    // If -nhl is set: No header in data, treat all lines as data.
    // If neither: Is the first line a header?
    // Usually CLI tools assume no header unless specified, OR assume first line is header.
    // "rcol reads the complete input... -header='...' Headerline, if the text has no headers, you can define headers."
    // This suggests the input might NOT have headers by default.
    // But -nhl says "The data contains no headline". This implies the DEFAULT is that data MIGHT have a headline?
    // Let's look at -rh "RemoveHeader removes the first line."
    // If -rh is used, we drop the first line.
    
    // Let's assume:
    // If -header is set, we use it.
    // If -rh is set, we skip the first line of input.
    // If -nhl is set, we treat all (remaining) lines as data.
    // If neither -header nor -nhl is set, do we treat first line as header?
    // Most likely, rcol treats all input as data unless told otherwise, OR it treats first line as header if not told -nhl.
    // Given -nhl exists, it strongly suggests the default is "Expect Headline".
    // So: Default = First line is header.
    // -nhl = No header in input (all lines are data).
    // -header = Use this string as header.
    // -rh = Remove first line (maybe it was a bad header?).
    
    let line_iter = filtered_lines.into_iter();
    


    // Handle input lines
    let mut first_line = true;
    for line in line_iter {
        if first_line {
            first_line = false;
            if args.rh {
                continue; // Remove first line
            }
            if args.header.is_none() && !args.nhl {
                // Treat first line as header
                let parts: Vec<String> = sep_regex.split(&line).map(|s| s.to_string()).collect();
                headers = parts;
                continue;
            }
        }
        
        let parts: Vec<String> = sep_regex.split(&line).map(|s| s.to_string()).collect();
        rows.push(parts);
    }

    // 3. Column Selection & Reordering
    // Parse column specs from args.columns
    let mut col_indices: Vec<usize> = Vec::new();
    if !args.columns.is_empty() {
        for col_spec in &args.columns {
            if col_spec.contains(':') {
                // Range
                let parts: Vec<&str> = col_spec.split(':').collect();
                if parts.len() == 2 {
                    let start: usize = parts[0].parse().map_err(|_| format!("Invalid range start: {}", parts[0]))?;
                    let end: usize = parts[1].parse().map_err(|_| format!("Invalid range end: {}", parts[1]))?;
                    // 1-based to 0-based
                    if start == 0 || end == 0 {
                        return Err("Column numbers must be 1-based".to_string());
                    }
                    if start <= end {
                        for i in start..=end {
                            col_indices.push(i - 1);
                        }
                    } else {
                        // Reverse range? "To rearrange the columns the columns can given in the wanted order."
                        // Usually ranges are low:high. But if user wants 3:1, maybe?
                        // Let's support reverse ranges if start > end.
                        let mut i = start;
                        while i >= end {
                            col_indices.push(i - 1);
                            if i == 0 { break; } // Should not happen due to check above
                            i -= 1;
                        }
                    }
                } else {
                    return Err(format!("Invalid range format: {}", col_spec));
                }
            } else {
                // Single number
                let idx: usize = col_spec.parse().map_err(|_| format!("Invalid column number: {}", col_spec))?;
                if idx == 0 {
                    return Err("Column numbers must be 1-based".to_string());
                }
                col_indices.push(idx - 1);
            }
        }
    } else {
        // Default: all columns.
        // We need to know max columns to select all.
        // We can check the first row or header.
        let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let header_cols = headers.len();
        let count = std::cmp::max(max_cols, header_cols);
        for i in 0..count {
            col_indices.push(i);
        }
    }

    // Apply selection to headers and rows
    let mut new_headers = Vec::new();
    for &idx in &col_indices {
        if idx < headers.len() {
            new_headers.push(headers[idx].clone());
        } else {
            new_headers.push("".to_string());
        }
    }
    headers = new_headers;

    // Handle explicit header argument (applied to OUTPUT columns)
    if let Some(h) = &args.header {
        let mut parts: Vec<String> = sep_regex.split(h).map(|s| s.to_string()).collect();
        // Adjust length to match output columns
        if parts.len() < col_indices.len() {
            parts.resize(col_indices.len(), "".to_string());
        } else if parts.len() > col_indices.len() {
            parts.truncate(col_indices.len());
        }
        headers = parts;
    }

    let mut new_rows = Vec::new();
    for row in rows {
        let mut new_row = Vec::new();
        for &idx in &col_indices {
            if idx < row.len() {
                new_row.push(row[idx].clone());
            } else {
                new_row.push("".to_string());
            }
        }
        new_rows.push(new_row);
    }
    rows = new_rows;

    // 4. Sorting
    if let Some(sort_col) = args.sortcol {
        // sort_col is 1-based output column number
        if sort_col > 0 && sort_col <= col_indices.len() {
            let idx = sort_col - 1;
            // Check if numeric sort is needed?
            // "Number refers to the number of the output column."
            // Usually text sort unless specified otherwise.
            // Requirement doesn't explicitly say numeric sort, but "-nn no numerical don't format numerical content right adjusted"
            // implies numerical detection.
            // For sorting, let's stick to string sort for now, or try numeric if it looks like number?
            // Simple string sort is safer unless we want to be fancy.
            rows.sort_by(|a, b| {
                let val_a = &a[idx];
                let val_b = &b[idx];
                // Try numeric sort if both are numbers?
                if let (Ok(num_a), Ok(num_b)) = (val_a.parse::<f64>(), val_b.parse::<f64>()) {
                    num_a.partial_cmp(&num_b).unwrap_or(Ordering::Equal)
                } else {
                    val_a.cmp(val_b)
                }
            });
        }
    }

    // 5. Grouping
    if let Some(gcol) = args.gcol {
        if gcol > 0 && gcol <= col_indices.len() {
            let idx = gcol - 1;
            let mut last_val = String::new();
            // We need to iterate and modify.
            // But we also need to insert separators?
            // "write a separator when the value in this column is different to the value in the previous line"
            // Wait, "write a separator" - does it mean insert a row? Or just visual separator?
            // "In the grouped column the second and all following lines of a group get the value '""'."
            // This implies modifying the data.
            // "write a separator" might mean a blank line or a line with dashes?
            // Usually in these tools it means a blank line or a specific separator line.
            // Let's assume it means inserting a separator row OR just modifying the values.
            // "write a separator... In the grouped column..."
            // It seems to imply TWO things:
            // 1. Separator between groups.
            // 2. Hiding repeated values.
            
            // Let's implement hiding repeated values first.
            // And for separator, maybe insert a special row? Or handle in formatter?
            // If I insert a row here, it complicates the TableData structure (which expects uniform columns).
            // Maybe I should add a `is_separator` flag to rows?
            // Or just let the formatter handle it?
            // But `process_input` returns `TableData`.
            // Let's modify `TableData` to support separator rows?
            // Or just insert an empty row?
            
            // "write a separator... to group the values"
            // Let's insert an empty row (all empty strings) between groups.
            
            let mut grouped_rows = Vec::new();
            let mut first = true;
            
            for mut row in rows {
                let val = row[idx].clone();
                if !first && val != last_val {
                    // Group change
                    // Insert separator row?
                    // Let's insert a row of empty strings.
                    let empty_row = vec!["".to_string(); row.len()];
                    grouped_rows.push(empty_row);
                }
                
                if !first && val == last_val && !args.gcolval {
                    // Hide value
                    row[idx] = "".to_string();
                }
                
                last_val = val;
                grouped_rows.push(row);
                first = false;
            }
            rows = grouped_rows;
        }
    }

    Ok(TableData { headers, rows, original_column_indices: col_indices })
}
