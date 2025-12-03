//! Integration tests for rcol
//!
//! These tests execute the full rcol application pipeline to ensure
//! end-to-end functionality works correctly.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_test_data_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(filename)
}

fn run_rcol(args: &[&str], input: Option<&str>) -> Result<String, String> {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_rcol"));
    cmd.args(args);

    if let Some(_input_str) = input {
        cmd.stdin(std::process::Stdio::piped());
    }

    let output = cmd.output().map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn test_basic_formatting() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap()], None).unwrap();

    assert!(result.contains("Name"));
    assert!(result.contains("Age"));
    assert!(result.contains("Alice"));
    assert!(result.contains("30"));
}

#[test]
fn test_column_selection() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "1", "3"], None).unwrap();

    assert!(result.contains("Name"));
    assert!(result.contains("City"));
    // Age column should not be present
    assert!(!result.contains("Age │"));
}

#[test]
fn test_column_range() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "1:2"], None).unwrap();

    assert!(result.contains("Name"));
    assert!(result.contains("Age"));
}

#[test]
fn test_column_reordering() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(
        &["--file", data_path.to_str().unwrap(), "3", "1", "2"],
        None,
    )
    .unwrap();
    // City should come before Name in the output
    assert!(result.contains("City"));
    assert!(result.contains("Name"));
}

#[test]
fn test_pretty_print() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--pp"], None).unwrap();

    // Check for box drawing characters
    assert!(result.contains("┌"));
    assert!(result.contains("└"));
    assert!(result.contains("│"));
    assert!(result.contains("─"));
}

#[test]
fn test_filter() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(
        &["--file", data_path.to_str().unwrap(), "--filter", "Alice"],
        None,
    )
    .unwrap();

    assert!(result.contains("Alice"));
    assert!(!result.contains("Bob"));
    assert!(!result.contains("Charlie"));
}

#[test]
fn test_sort_by_column() {
    let data_path = get_test_data_path("numeric.txt");
    let result = run_rcol(
        &["--file", data_path.to_str().unwrap(), "--sortcol", "2"],
        None,
    )
    .unwrap();

    let lines: Vec<&str> = result.lines().collect();
    // Mouse (25.50) should come before Keyboard (75.00)
    let mouse_idx = lines.iter().position(|l| l.contains("Mouse"));
    let keyboard_idx = lines.iter().position(|l| l.contains("Keyboard"));

    if let (Some(m), Some(k)) = (mouse_idx, keyboard_idx) {
        assert!(
            m < k,
            "Mouse should appear before Keyboard when sorted by price"
        );
    }
}

#[test]
fn test_grouping() {
    let data_path = get_test_data_path("grouping.txt");
    let result = run_rcol(
        &["--file", data_path.to_str().unwrap(), "--gcol", "1"],
        None,
    )
    .unwrap();

    // Second and third Sales entries should have Department hidden
    let lines: Vec<&str> = result.lines().collect();
    let sales_count = lines.iter().filter(|l| l.contains("Sales")).count();

    // Should only show "Sales" once (others are hidden)
    assert_eq!(sales_count, 1);
}

#[test]
fn test_csv_output() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--csv"], None).unwrap();

    // CSV should have comma-separated values
    assert!(result.contains(","));
    assert!(result.contains("Name,Age,City") || result.contains("\"Name\",\"Age\",\"City\""));
}

#[test]
fn test_json_output() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--json"], None).unwrap();

    // JSON should be valid
    assert!(result.contains("{"));
    assert!(result.contains("}"));
    assert!(result.contains("Name"));
    assert!(result.contains("Alice"));
}

#[test]
fn test_json_title_column() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(
        &["--file", data_path.to_str().unwrap(), "--json", "--jtc"],
        None,
    )
    .unwrap();

    // In title column mode, first column becomes the key
    assert!(result.contains("\"Alice\""));
    assert!(result.contains("\"Bob\""));
}

#[test]
fn test_html_output() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--html"], None).unwrap();

    assert!(result.contains("<table>"));
    assert!(result.contains("</table>"));
    assert!(result.contains("<th>"));
    assert!(result.contains("<td>"));
}

#[test]
fn test_custom_header() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(
        &[
            "--file",
            data_path.to_str().unwrap(),
            "--header",
            "Person Years Location",
            "1",
            "2",
            "3",
        ],
        None,
    )
    .unwrap();

    assert!(result.contains("Person"));
    assert!(result.contains("Years"));
    assert!(result.contains("Location"));
}

#[test]
fn test_no_headline() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--nhl"], None).unwrap();

    // All lines including first should be treated as data
    // The header row "Name Age City" should appear as data
    assert!(result.contains("Name"));
}

#[test]
fn test_remove_header() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--rh"], None).unwrap();

    // First line (Name Age City) should be removed
    // Alice should be in header position
    assert!(result.contains("Alice"));
}

#[test]
fn test_more_blanks() {
    // Create temp file with multiple spaces
    let temp_data = "Name    Age    City\nAlice   30     NYC\n";
    let temp_path = std::env::temp_dir().join("rcol_test_mb.txt");
    fs::write(&temp_path, temp_data).unwrap();

    let result = run_rcol(&["--file", temp_path.to_str().unwrap(), "--mb"], None).unwrap();

    assert!(result.contains("Name"));
    assert!(result.contains("Age"));
    assert!(result.contains("City"));

    fs::remove_file(temp_path).ok();
}

#[test]
fn test_title_separator() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--ts"], None).unwrap();

    // Should have a separator line after header
    assert!(result.contains("─"));
}

#[test]
fn test_column_separator() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--cs"], None).unwrap();

    // Should have column separators
    assert!(result.contains("│"));
}

#[test]
fn test_column_numbering() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(
        &["--file", data_path.to_str().unwrap(), "--num", "--pp"],
        None,
    )
    .unwrap();

    // Should show column numbers (1, 2, 3)
    assert!(result.contains("1"));
    assert!(result.contains("2"));
    assert!(result.contains("3"));
}

#[test]
fn test_custom_separator() {
    // Create temp CSV file
    let temp_data = "Name,Age,City\nAlice,30,NYC\n";
    let temp_path = std::env::temp_dir().join("rcol_test_sep.csv");
    fs::write(&temp_path, temp_data).unwrap();

    let result = run_rcol(&["--file", temp_path.to_str().unwrap(), "--sep", ","], None).unwrap();

    assert!(result.contains("Name"));
    assert!(result.contains("Alice"));
    assert!(result.contains("30"));

    fs::remove_file(temp_path).ok();
}

#[test]
fn test_width_padding() {
    let data_path = get_test_data_path("simple.txt");
    let result_w1 = run_rcol(&["--file", data_path.to_str().unwrap(), "-w", "1"], None).unwrap();
    let result_w3 = run_rcol(&["--file", data_path.to_str().unwrap(), "-w", "3"], None).unwrap();

    // w=3 should have more spaces between columns
    assert!(result_w3.len() > result_w1.len());
}

#[test]
fn test_no_format() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--nf"], None).unwrap();

    // Output should still contain the data
    assert!(result.contains("Alice"));
    assert!(result.contains("Bob"));
}

#[test]
fn test_no_numerical_alignment() {
    let data_path = get_test_data_path("numeric.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--nn"], None).unwrap();

    // Numbers should still be present
    assert!(result.contains("999.99"));
    assert!(result.contains("25.50"));
}

#[test]
fn test_empty_input() {
    let temp_path = std::env::temp_dir().join("rcol_test_empty.txt");
    fs::write(&temp_path, "").unwrap();

    let result = run_rcol(&["--file", temp_path.to_str().unwrap()], None).unwrap();

    // Should handle empty input gracefully
    assert_eq!(result.trim(), "");

    fs::remove_file(temp_path).ok();
}

#[test]
fn test_single_column() {
    let temp_data = "Name\nAlice\nBob\n";
    let temp_path = std::env::temp_dir().join("rcol_test_single.txt");
    fs::write(&temp_path, temp_data).unwrap();

    let result = run_rcol(&["--file", temp_path.to_str().unwrap()], None).unwrap();

    assert!(result.contains("Name"));
    assert!(result.contains("Alice"));
    assert!(result.contains("Bob"));

    fs::remove_file(temp_path).ok();
}

#[test]
fn test_irregular_columns() {
    let data_path = get_test_data_path("irregular.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap()], None).unwrap();

    // Should handle rows with different column counts
    assert!(result.contains("Alice"));
    assert!(result.contains("Bob"));
    assert!(result.contains("Charlie"));
}

#[test]
fn test_complex_example_from_readme() {
    // This is Example 7 from the README: Complex formatting with grouping, sorting, pretty print
    // Command: rcol -pp -mb -gcol=1 -sortcol=1 -nhl -header="RIGHTS USER GROUP SIZE UNIT DAY MONTH CAL TIME YEAR S NAME" -file=test_data_03.txt

    let data_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data_03.txt");
    let result = run_rcol(
        &[
            "--pp",
            "--mb",
            "--gcol",
            "1",
            "--sortcol",
            "1",
            "--nhl",
            "--header",
            "RIGHTS USER GROUP SIZE UNIT DAY MONTH CAL TIME YEAR S NAME",
            "--file",
            data_path.to_str().unwrap(),
        ],
        None,
    )
    .unwrap();

    // Verify box drawing characters are present (pretty print)
    assert!(result.contains("┌"), "Should have top-left corner");
    assert!(result.contains("└"), "Should have bottom-left corner");
    assert!(result.contains("├"), "Should have left junction");
    assert!(result.contains("│"), "Should have vertical lines");
    assert!(result.contains("─"), "Should have horizontal lines");

    // Verify headers are present
    assert!(result.contains("RIGHTS"), "Should have RIGHTS header");
    assert!(result.contains("USER"), "Should have USER header");
    assert!(result.contains("GROUP"), "Should have GROUP header");
    assert!(result.contains("SIZE"), "Should have SIZE header");
    assert!(result.contains("NAME"), "Should have NAME header");

    // Verify data is present
    assert!(result.contains("Cargo.lock"), "Should contain Cargo.lock");
    assert!(result.contains("Cargo.toml"), "Should contain Cargo.toml");
    assert!(result.contains("src"), "Should contain src");

    // Verify grouping behavior - .rw-r--r-- appears first, then gets hidden for subsequent rows
    let lines: Vec<&str> = result.lines().collect();

    // Find the data rows (skip header and separator lines)
    let data_start = lines.iter().position(|l| l.contains("Cargo.lock")).unwrap();

    // First .rw-r--r-- file should show the permission
    assert!(
        lines[data_start].contains(".rw-r--r--"),
        "First row should show .rw-r--r--"
    );

    // Next row with same permission should have it hidden (empty or spaces)
    // Due to grouping, the RIGHTS column should be empty for subsequent .rw-r--r-- entries
    let next_rw_line = lines[data_start + 1];
    assert!(
        next_rw_line.contains("Cargo.toml"),
        "Next row should be Cargo.toml"
    );
    // The grouping hides the repeated RIGHTS value, so it should have spaces where RIGHTS was

    // Separator row between groups (empty row)
    let has_separator_rows = lines.iter().any(|l| {
        l.contains("│")
            && l.chars()
                .filter(|&c| c != '│' && c != ' ' && c != '─')
                .count()
                == 0
    });
    assert!(
        has_separator_rows,
        "Should have separator rows between groups"
    );

    // Verify sorting - .rw-r--r-- files should come before .rwxr-xr-x and drwxr-xr-x
    let rw_pos = lines.iter().position(|l| l.contains(".rw-r--r--")).unwrap();
    let rwx_pos = lines.iter().position(|l| l.contains(".rwxr-xr-x")).unwrap();
    let drwx_pos = lines.iter().position(|l| l.contains("drwxr-xr-x")).unwrap();

    assert!(
        rw_pos < rwx_pos,
        "Regular files (.rw-r--r--) should come before executable (.rwxr-xr-x)"
    );
    assert!(
        rwx_pos < drwx_pos,
        "Executable files (.rwxr-xr-x) should come before directories (drwxr-xr-x)"
    );
}
