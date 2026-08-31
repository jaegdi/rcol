use crate::args::AppArgs;
use regex::Regex;
use std::cmp::Ordering;

/// Represents processed tabular data with headers and rows.
#[derive(Debug)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub original_column_indices: Vec<usize>,
}

/// Processes input lines according to application arguments to produce table data.
///
/// Pipeline:
/// 1. Extract/remove the header line first (so filtering never discards it).
/// 2. Split remaining lines into columns and apply the filter regex.
/// 3. Select and reorder columns.
/// 4. Apply custom header if provided.
/// 5. Sort rows by specified column.
/// 6. Group rows by specified column.
pub fn process_input(lines: Vec<String>, args: &AppArgs) -> Result<TableData, String> {
    let sep_regex = build_separator_regex(args);

    // 1. Header handling: extract or remove the first line before any filtering.
    let (mut headers, data_lines) = extract_header(lines, args, &sep_regex)?;

    // 2. Split and filter data lines.
    let mut rows = split_and_filter(data_lines, args, &sep_regex)?;

    // 3. Column selection & reordering.
    let col_indices = parse_column_specs(args, &headers, &rows)?;
    headers = select_headers(headers, &col_indices);
    rows = select_rows(rows, &col_indices);

    // 4. Apply explicit custom header (after selection so it matches output columns).
    if let Some(h) = &args.header {
        headers = apply_custom_header(h, &sep_regex, col_indices.len());
    }

    // 5. Sorting.
    if let Some(sort_col) = args.sortcol {
        sort_rows(&mut rows, sort_col, col_indices.len());
    }

    // 6. Grouping.
    if let Some(gcol) = args.gcol {
        rows = group_rows(rows, gcol, col_indices.len(), args.gcolval);
    }

    Ok(TableData {
        headers,
        rows,
        original_column_indices: col_indices,
    })
}

/// Builds the regex used to split input lines into columns.
fn build_separator_regex(args: &AppArgs) -> Regex {
    if args.mb {
        Regex::new(r"\s+").unwrap()
    } else {
        Regex::new(&regex::escape(&args.sep)).unwrap()
    }
}

/// Extracts the header from the input lines and returns the remaining data lines.
///
/// Priority:
/// - If `-header` is provided, use it as the header.
/// - If `-rh` is set, discard the first line.
/// - If `-nhl` is set, there is no header in input.
/// - Otherwise, the first line is treated as the header.
fn extract_header(
    lines: Vec<String>,
    args: &AppArgs,
    sep_regex: &Regex,
) -> Result<(Vec<String>, Vec<String>), String> {
    if lines.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut iter = lines.into_iter();
    let first = iter.next().unwrap();

    if args.rh {
        // First line removed; any explicit custom header is applied later.
        return Ok((Vec::new(), iter.collect()));
    }

    if args.nhl {
        // No header in input; keep all lines as data.
        let mut data_lines: Vec<String> = iter.collect();
        data_lines.insert(0, first);
        return Ok((Vec::new(), data_lines));
    }

    // Default: first line is the header.
    Ok((split_header(&first, sep_regex), iter.collect()))
}

/// Splits a header string into individual column names.
fn split_header(header: &str, sep_regex: &Regex) -> Vec<String> {
    sep_regex.split(header).map(|s| s.to_string()).collect()
}

/// Splits data lines into columns and applies the optional filter regex.
fn split_and_filter(
    lines: Vec<String>,
    args: &AppArgs,
    sep_regex: &Regex,
) -> Result<Vec<Vec<String>>, String> {
    let filter_regex = args
        .filter
        .as_ref()
        .map(|p| Regex::new(p).map_err(|e| format!("Invalid filter regex: {}", e)))
        .transpose()?;

    let mut rows = Vec::new();
    for line in lines {
        if let Some(re) = &filter_regex {
            if !re.is_match(&line) {
                continue;
            }
        }
        rows.push(sep_regex.split(&line).map(|s| s.to_string()).collect());
    }
    Ok(rows)
}

/// Parses column specifications from CLI arguments.
///
/// Supports individual indices (`1 3 5`) and ranges (`1:3`, `3:1`).
/// Returns 0-based indices.
fn parse_column_specs(
    args: &AppArgs,
    headers: &[String],
    rows: &[Vec<String>],
) -> Result<Vec<usize>, String> {
    if args.columns.is_empty() {
        let max_cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
        let count = std::cmp::max(max_cols, headers.len());
        return Ok((0..count).collect());
    }

    let mut col_indices = Vec::new();
    for col_spec in &args.columns {
        if col_spec.contains(':') {
            parse_range(col_spec, &mut col_indices)?;
        } else {
            parse_single(col_spec, &mut col_indices)?;
        }
    }
    Ok(col_indices)
}

/// Parses a single column index and appends its 0-based value.
fn parse_single(spec: &str, indices: &mut Vec<usize>) -> Result<(), String> {
    let idx: usize = spec
        .parse()
        .map_err(|_| format!("Invalid column number: {}", spec))?;
    if idx == 0 {
        return Err("Column numbers must be 1-based".to_string());
    }
    indices.push(idx - 1);
    Ok(())
}

/// Parses a column range and appends the corresponding 0-based indices.
fn parse_range(spec: &str, indices: &mut Vec<usize>) -> Result<(), String> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid range format: {}", spec));
    }
    let start: usize = parts[0]
        .parse()
        .map_err(|_| format!("Invalid range start: {}", parts[0]))?;
    let end: usize = parts[1]
        .parse()
        .map_err(|_| format!("Invalid range end: {}", parts[1]))?;
    if start == 0 || end == 0 {
        return Err("Column numbers must be 1-based".to_string());
    }

    if start <= end {
        for i in start..=end {
            indices.push(i - 1);
        }
    } else {
        let mut i = start;
        loop {
            indices.push(i - 1);
            if i == end {
                break;
            }
            i -= 1;
        }
    }
    Ok(())
}

/// Selects only the requested columns from the headers.
fn select_headers(headers: Vec<String>, col_indices: &[usize]) -> Vec<String> {
    col_indices
        .iter()
        .map(|&idx| {
            if idx < headers.len() {
                headers[idx].clone()
            } else {
                String::new()
            }
        })
        .collect()
}

/// Selects only the requested columns from each row.
fn select_rows(rows: Vec<Vec<String>>, col_indices: &[usize]) -> Vec<Vec<String>> {
    rows.into_iter()
        .map(|row| {
            col_indices
                .iter()
                .map(|&idx| {
                    if idx < row.len() {
                        row[idx].clone()
                    } else {
                        String::new()
                    }
                })
                .collect()
        })
        .collect()
}

/// Applies a user-provided custom header, padded/truncated to match output columns.
fn apply_custom_header(header: &str, sep_regex: &Regex, col_count: usize) -> Vec<String> {
    let mut parts: Vec<String> = sep_regex.split(header).map(|s| s.to_string()).collect();
    if parts.len() < col_count {
        parts.resize(col_count, String::new());
    } else if parts.len() > col_count {
        parts.truncate(col_count);
    }
    parts
}

/// Sorts rows by the specified 1-based output column index.
fn sort_rows(rows: &mut [Vec<String>], sort_col: usize, col_count: usize) {
    if sort_col == 0 || sort_col > col_count {
        return;
    }
    let idx = sort_col - 1;
    rows.sort_by(|a, b| {
        let val_a = &a[idx];
        let val_b = &b[idx];
        if let (Ok(num_a), Ok(num_b)) = (val_a.parse::<f64>(), val_b.parse::<f64>()) {
            num_a.partial_cmp(&num_b).unwrap_or(Ordering::Equal)
        } else {
            val_a.cmp(val_b)
        }
    });
}

/// Groups rows by the specified 1-based output column index.
///
/// Repeated group values are replaced with empty strings unless `keep_vals` is true.
/// An empty separator row is inserted between groups.
fn group_rows(
    rows: Vec<Vec<String>>,
    gcol: usize,
    col_count: usize,
    keep_vals: bool,
) -> Vec<Vec<String>> {
    if gcol == 0 || gcol > col_count {
        return rows;
    }
    let idx = gcol - 1;
    let mut grouped = Vec::new();
    let mut last_val = String::new();
    let mut first = true;

    for mut row in rows {
        let val = row[idx].clone();
        if !first && val != last_val {
            grouped.push(vec![String::new(); row.len()]);
        }
        if !first && val == last_val && !keep_vals {
            row[idx] = String::new();
        }
        last_val = val;
        grouped.push(row);
        first = false;
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_data_creation() {
        let data = TableData {
            headers: vec!["Col1".to_string(), "Col2".to_string()],
            rows: vec![
                vec!["A".to_string(), "B".to_string()],
                vec!["C".to_string(), "D".to_string()],
            ],
            original_column_indices: vec![0, 1],
        };

        assert_eq!(data.headers.len(), 2);
        assert_eq!(data.rows.len(), 2);
        assert_eq!(data.original_column_indices, vec![0, 1]);
    }

    #[test]
    fn test_process_simple_data() {
        let lines = vec![
            "Name Age".to_string(),
            "Alice 30".to_string(),
            "Bob 25".to_string(),
        ];

        let args = AppArgs::default();
        let result = process_input(lines, &args).unwrap();

        assert_eq!(result.headers, vec!["Name", "Age"]);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0], vec!["Alice", "30"]);
        assert_eq!(result.rows[1], vec!["Bob", "25"]);
    }

    #[test]
    fn test_filter_preserves_header() {
        let lines = vec![
            "Name Age".to_string(),
            "Alice 30".to_string(),
            "Bob 25".to_string(),
            "Charlie 35".to_string(),
        ];

        let mut args = AppArgs::default();
        args.filter = Some("Bob".to_string());

        let result = process_input(lines, &args).unwrap();

        // Header should be preserved even though it doesn't match the filter.
        assert_eq!(result.headers, vec!["Name", "Age"]);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0], vec!["Bob", "25"]);
    }

    #[test]
    fn test_process_with_custom_header() {
        let lines = vec!["Alice 30".to_string(), "Bob 25".to_string()];

        let mut args = AppArgs::default();
        args.header = Some("Name Age".to_string());
        args.nhl = true;

        let result = process_input(lines, &args).unwrap();

        assert_eq!(result.headers, vec!["Name", "Age"]);
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_process_column_selection() {
        let lines = vec![
            "Name Age City".to_string(),
            "Alice 30 NYC".to_string(),
            "Bob 25 LA".to_string(),
        ];

        let mut args = AppArgs::default();
        args.columns = vec!["1".to_string(), "3".to_string()];

        let result = process_input(lines, &args).unwrap();

        assert_eq!(result.headers, vec!["Name", "City"]);
        assert_eq!(result.rows[0], vec!["Alice", "NYC"]);
        assert_eq!(result.rows[1], vec!["Bob", "LA"]);
    }

    #[test]
    fn test_process_column_range() {
        let lines = vec!["A B C D".to_string(), "1 2 3 4".to_string()];

        let mut args = AppArgs::default();
        args.columns = vec!["2:4".to_string()];

        let result = process_input(lines, &args).unwrap();

        assert_eq!(result.headers, vec!["B", "C", "D"]);
        assert_eq!(result.rows[0], vec!["2", "3", "4"]);
    }

    #[test]
    fn test_process_column_reorder() {
        let lines = vec!["A B C".to_string(), "1 2 3".to_string()];

        let mut args = AppArgs::default();
        args.columns = vec!["3".to_string(), "1".to_string(), "2".to_string()];

        let result = process_input(lines, &args).unwrap();

        assert_eq!(result.headers, vec!["C", "A", "B"]);
        assert_eq!(result.rows[0], vec!["3", "1", "2"]);
    }

    #[test]
    fn test_process_sorting_numeric() {
        let lines = vec![
            "Name Value".to_string(),
            "C 300".to_string(),
            "A 100".to_string(),
            "B 200".to_string(),
        ];

        let mut args = AppArgs::default();
        args.sortcol = Some(2);

        let result = process_input(lines, &args).unwrap();

        assert_eq!(result.rows[0][1], "100");
        assert_eq!(result.rows[1][1], "200");
        assert_eq!(result.rows[2][1], "300");
    }

    #[test]
    fn test_process_sorting_text() {
        let lines = vec![
            "Name Age".to_string(),
            "Charlie 30".to_string(),
            "Alice 25".to_string(),
            "Bob 35".to_string(),
        ];

        let mut args = AppArgs::default();
        args.sortcol = Some(1);

        let result = process_input(lines, &args).unwrap();

        assert_eq!(result.rows[0][0], "Alice");
        assert_eq!(result.rows[1][0], "Bob");
        assert_eq!(result.rows[2][0], "Charlie");
    }

    #[test]
    fn test_process_grouping() {
        let lines = vec![
            "Dept Name".to_string(),
            "Sales Alice".to_string(),
            "Sales Bob".to_string(),
            "Engineering Charlie".to_string(),
        ];

        let mut args = AppArgs::default();
        args.gcol = Some(1);

        let result = process_input(lines, &args).unwrap();

        assert_eq!(result.rows[0][0], "Sales");
        assert_eq!(result.rows[1][0], ""); // Hidden
        assert_eq!(result.rows[3][0], "Engineering");
    }

    #[test]
    fn test_process_with_mb() {
        let lines = vec!["Name    Age".to_string(), "Alice   30".to_string()];

        let mut args = AppArgs::default();
        args.mb = true;

        let result = process_input(lines, &args).unwrap();

        assert_eq!(result.headers, vec!["Name", "Age"]);
        assert_eq!(result.rows[0], vec!["Alice", "30"]);
    }

    #[test]
    fn test_process_remove_header() {
        let lines = vec![
            "Skip this line".to_string(),
            "Name Age".to_string(),
            "Alice 30".to_string(),
        ];

        let mut args = AppArgs::default();
        args.rh = true;
        args.nhl = true;
        args.header = Some("Name Age".to_string());

        let result = process_input(lines, &args).unwrap();

        assert_eq!(result.headers, vec!["Name", "Age"]);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0], vec!["Name", "Age"]);
        assert_eq!(result.rows[1], vec!["Alice", "30"]);
    }

    #[test]
    fn test_process_no_headline() {
        let lines = vec!["Alice 30".to_string(), "Bob 25".to_string()];

        let mut args = AppArgs::default();
        args.nhl = true;
        args.header = Some("Name Age".to_string());

        let result = process_input(lines, &args).unwrap();

        assert_eq!(result.headers, vec!["Name", "Age"]);
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_process_empty_input() {
        let lines = vec![];
        let args = AppArgs::default();

        let result = process_input(lines, &args).unwrap();

        assert!(result.headers.is_empty());
        assert!(result.rows.is_empty());
    }
}
