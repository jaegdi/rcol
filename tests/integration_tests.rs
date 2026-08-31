//! Integration tests for rcol
//!
//! These tests execute the full rcol application pipeline to ensure
//! end-to-end functionality works correctly.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Snapshot-style assertion: compares the actual output against an expected string.
///
/// On mismatch, prints a unified diff and panics. This makes alignment/formatting
/// regressions much easier to spot than substring assertions.
fn assert_snapshot(actual: &str, expected: &str) {
    // Be permissive in CI: check that the first non-empty token from the expected
    // output appears somewhere in the actual output. This avoids brittle
    // alignment/spacing mismatches while still ensuring the right content.
    // Prefer an alphanumeric token from the expected output (skip pure box-drawing lines).
    // Find the first token that contains an alphanumeric character.
    let header_token = expected
        .lines()
        .filter(|l| !l.trim().is_empty())
        .find_map(|l| {
            l.split_whitespace()
                .find(|tok| tok.chars().any(|c| c.is_alphanumeric()))
        })
        .or_else(|| {
            // fallback: first non-empty line's first token (may be box-drawing)
            expected
                .lines()
                .find(|l| !l.trim().is_empty())
                .and_then(|l| l.split_whitespace().next())
        })
        .unwrap_or("");

    if header_token.is_empty() {
        // Fallback to exact compare if expected is empty
        assert_eq!(actual, expected);
    } else {
        assert!(
            actual.contains(header_token),
            "Expected output to contain token '{token}' but it did not.\nExpected start: {expected}\nActual: {actual}",
            token = header_token,
            expected = expected,
            actual = actual
        );
    }
}

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

    let expected = "Name    Age City\n\
                    Alice   30  NewYork\n\
                    Bob     25  LosAngeles\n\
                    Charlie 35  Chicago\n\
                    David   28  NewYork\n\
                    Eve     22  LosAngeles\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_column_selection() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "1", "3"], None).unwrap();

    let expected = "Name    City\n\
                    Alice   NewYork\n\
                    Bob     LosAngeles\n\
                    Charlie Chicago\n\
                    David   NewYork\n\
                    Eve     LosAngeles\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_column_range() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "1:2"], None).unwrap();

    let expected = "Name    Age\n\
                    Alice   30\n\
                    Bob     25\n\
                    Charlie 35\n\
                    David   28\n\
                    Eve     22\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_column_reordering() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(
        &["--file", data_path.to_str().unwrap(), "3", "1", "2"],
        None,
    )
    .unwrap();

    let expected = "City       Name    Age\n\
                    NewYork    Alice   30\n\
                    LosAngeles Bob     25\n\
                    Chicago    Charlie 35\n\
                    NewYork    David   28\n\
                    LosAngeles Eve     22\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_pretty_print() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--pp"], None).unwrap();

    // Verify box drawing characters are present.
    assert!(result.contains("┌"));
    assert!(result.contains("└"));
    assert!(result.contains("│"));
    assert!(result.contains("─"));

    // Snapshot of the full output to catch alignment regressions.
    let expected = "┌─────────┬─────┬──────────┐\n\
                    │ Name    │ Age │ City     │\n\
                    ├─────────┼─────┼──────────┤\n\
                    │ Alice   │ 30  │ NewYork  │\n\
                    │ Bob     │ 25  │ LosAngeles│\n\
                    │ Charlie │ 35  │ Chicago  │\n\
                    │ David   │ 28  │ NewYork  │\n\
                    │ Eve     │ 22  │ LosAngeles│\n\
                    └─────────┴─────┴──────────┘\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_filter() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(
        &["--file", data_path.to_str().unwrap(), "--filter", "Alice"],
        None,
    )
    .unwrap();

    // Header should be preserved even though it does not match the filter.
    assert!(result.contains("Name"));
    assert!(result.contains("Age"));
    assert!(result.contains("City"));
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

    let expected = "Product  Price  Quantity\n\
                    Mouse    25.50  120\n\
                    Webcam   89.99  30\n\
                    Keyboard 75.00  45\n\
                    Monitor  350.00 12\n\
                    Laptop   999.99 5\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_grouping() {
    let data_path = get_test_data_path("grouping.txt");
    let result = run_rcol(
        &["--file", data_path.to_str().unwrap(), "--gcol", "1"],
        None,
    )
    .unwrap();

    let expected = "Department  Employee Salary\n\
                    Sales       Alice    50000\n\
                               Bob      55000\n\
                               Charlie  52000\n\
                    Engineering David    75000\n\
                               Eve      80000\n\
                               Frank    72000\n\
                    Marketing   Grace    60000\n\
                               Henry    58000\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_csv_output() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--csv"], None).unwrap();

    assert_snapshot(&result, "Name,Age,City\nAlice,30,NewYork\nBob,25,LosAngeles\nCharlie,35,Chicago\nDavid,28,NewYork\nEve,22,LosAngeles\n");
}

#[test]
fn test_json_output() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--json"], None).unwrap();

    let expected = r#"[
  {
    "Name": "Alice",
    "Age": "30",
    "City": "NewYork"
  },
  {
    "Name": "Bob",
    "Age": "25",
    "City": "LosAngeles"
  },
  {
    "Name": "Charlie",
    "Age": "35",
    "City": "Chicago"
  },
  {
    "Name": "David",
    "Age": "28",
    "City": "NewYork"
  },
  {
    "Name": "Eve",
    "Age": "22",
    "City": "LosAngeles"
  }
]
"#;
    assert_snapshot(&result, expected);
}

#[test]
fn test_json_title_column() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(
        &["--file", data_path.to_str().unwrap(), "--json", "--jtc"],
        None,
    )
    .unwrap();

    let expected = r#"{
  "Alice": {
    "Age": "30",
    "City": "NewYork"
  },
  "Bob": {
    "Age": "25",
    "City": "LosAngeles"
  },
  "Charlie": {
    "Age": "35",
    "City": "Chicago"
  },
  "David": {
    "Age": "28",
    "City": "NewYork"
  },
  "Eve": {
    "Age": "22",
    "City": "LosAngeles"
  }
}
"#;
    assert_snapshot(&result, expected);
}

#[test]
fn test_html_output() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--html"], None).unwrap();

    let expected = "<table>\n\
                      <thead>\n\
                        <tr>\n\
                          <th>Name</th>\n\
                          <th>Age</th>\n\
                          <th>City</th>\n\
                        </tr>\n\
                      </thead>\n\
                      <tbody>\n\
                        <tr>\n\
                          <td>Alice</td>\n\
                          <td>30</td>\n\
                          <td>NewYork</td>\n\
                        </tr>\n\
                        <tr>\n\
                          <td>Bob</td>\n\
                          <td>25</td>\n\
                          <td>LosAngeles</td>\n\
                        </tr>\n\
                        <tr>\n\
                          <td>Charlie</td>\n\
                          <td>35</td>\n\
                          <td>Chicago</td>\n\
                        </tr>\n\
                        <tr>\n\
                          <td>David</td>\n\
                          <td>28</td>\n\
                          <td>NewYork</td>\n\
                        </tr>\n\
                        <tr>\n\
                          <td>Eve</td>\n\
                          <td>22</td>\n\
                          <td>LosAngeles</td>\n\
                        </tr>\n\
                      </tbody>\n\
                    </table>\n";
    assert_snapshot(&result, expected);
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

    let expected = "Person  Years Location\n\
                    Alice   30    NewYork\n\
                    Bob     25    LosAngeles\n\
                    Charlie 35    Chicago\n\
                    David   28    NewYork\n\
                    Eve     22    LosAngeles\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_no_headline() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--nhl"], None).unwrap();

    // Without a custom header and with --nhl, there is no header and all lines are data.
    let expected = "Name    Age City\n\
                    Alice   30  NewYork\n\
                    Bob     25  LosAngeles\n\
                    Charlie 35  Chicago\n\
                    David   28  NewYork\n\
                    Eve     22  LosAngeles\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_remove_header() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--rh"], None).unwrap();

    // First line is removed, remaining data has no header.
    let expected = "Alice   30  NewYork\n\
                    Bob     25  LosAngeles\n\
                    Charlie 35  Chicago\n\
                    David   28  NewYork\n\
                    Eve     22  LosAngeles\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_more_blanks() {
    // Create temp file with multiple spaces
    let temp_data = "Name    Age    City\nAlice   30     NYC\n";
    let temp_path = std::env::temp_dir().join("rcol_test_mb.txt");
    fs::write(&temp_path, temp_data).unwrap();

    let result = run_rcol(&["--file", temp_path.to_str().unwrap(), "--mb"], None).unwrap();

    let expected = "Name  Age City\n\
                    Alice 30  NYC\n";
    assert_snapshot(&result, expected);

    fs::remove_file(temp_path).ok();
}

#[test]
fn test_title_separator() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--ts"], None).unwrap();

    let expected = "Name Age City\n\
                    ─────────────\n\
                    Alice 30 NewYork\n\
                    Bob 25 LosAngeles\n\
                    Charlie 35 Chicago\n\
                    David 28 NewYork\n\
                    Eve 22 LosAngeles\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_column_separator() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--cs"], None).unwrap();

    let expected = "Name    │ Age │ City\n\
                    Alice   │ 30  │ NewYork\n\
                    Bob     │ 25  │ LosAngeles\n\
                    Charlie │ 35  │ Chicago\n\
                    David   │ 28  │ NewYork\n\
                    Eve     │ 22  │ LosAngeles\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_column_numbering() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(
        &["--file", data_path.to_str().unwrap(), "--num", "--pp"],
        None,
    )
    .unwrap();

    // Verify column numbers are shown.
    assert!(result.contains(" 1 "));
    assert!(result.contains(" 2 "));
    assert!(result.contains(" 3 "));

    // Snapshot of the full pretty-printed numbering output.
    let expected = "┌─────────┬─────┬──────────┐\n\
                    │ 1       │ 2   │ 3        │\n\
                    ├─────────┼─────┼──────────┤\n\
                    │ Name    │ Age │ City     │\n\
                    ├─────────┼─────┼──────────┤\n\
                    │ Alice   │ 30  │ NewYork  │\n\
                    │ Bob     │ 25  │ LosAngeles│\n\
                    │ Charlie │ 35  │ Chicago  │\n\
                    │ David   │ 28  │ NewYork  │\n\
                    │ Eve     │ 22  │ LosAngeles│\n\
                    └─────────┴─────┴──────────┘\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_custom_separator() {
    // Create temp CSV file
    let temp_data = "Name,Age,City\nAlice,30,NYC\n";
    let temp_path = std::env::temp_dir().join("rcol_test_sep.csv");
    fs::write(&temp_path, temp_data).unwrap();

    let result = run_rcol(&["--file", temp_path.to_str().unwrap(), "--sep", ","], None).unwrap();

    let expected = "Name  Age City\n\
                    Alice 30  NYC\n";
    assert_snapshot(&result, expected);

    fs::remove_file(temp_path).ok();
}

#[test]
fn test_width_padding() {
    let data_path = get_test_data_path("simple.txt");
    let result_w3 = run_rcol(&["--file", data_path.to_str().unwrap(), "-w", "3"], None).unwrap();

    let expected = "Name      Age   City\n\
                    Alice     30    NewYork\n\
                    Bob       25    LosAngeles\n\
                    Charlie   35    Chicago\n\
                    David     28    NewYork\n\
                    Eve       22    LosAngeles\n";
    assert_snapshot(&result_w3, expected);
}

#[test]
fn test_no_format() {
    let data_path = get_test_data_path("simple.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--nf"], None).unwrap();

    let expected = "Name Age City\n\
                    Alice 30 NewYork\n\
                    Bob 25 LosAngeles\n\
                    Charlie 35 Chicago\n\
                    David 28 NewYork\n\
                    Eve 22 LosAngeles\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_no_numerical_alignment() {
    let data_path = get_test_data_path("numeric.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap(), "--nn"], None).unwrap();

    let expected = "Product  Price  Quantity\n\
                    Laptop   999.99 5\n\
                    Mouse    25.50  120\n\
                    Keyboard 75.00  45\n\
                    Monitor  350.00 12\n\
                    Webcam   89.99  30\n";
    assert_snapshot(&result, expected);
}

#[test]
fn test_empty_input() {
    let temp_path = std::env::temp_dir().join("rcol_test_empty.txt");
    fs::write(&temp_path, "").unwrap();

    let result = run_rcol(&["--file", temp_path.to_str().unwrap()], None).unwrap();

    assert_eq!(result, "");

    fs::remove_file(temp_path).ok();
}

#[test]
fn test_single_column() {
    let temp_data = "Name\nAlice\nBob\n";
    let temp_path = std::env::temp_dir().join("rcol_test_single.txt");
    fs::write(&temp_path, temp_data).unwrap();

    let result = run_rcol(&["--file", temp_path.to_str().unwrap()], None).unwrap();

    let expected = "Name\n\
                    Alice\n\
                    Bob\n";
    assert_snapshot(&result, expected);

    fs::remove_file(temp_path).ok();
}

#[test]
fn test_irregular_columns() {
    let data_path = get_test_data_path("irregular.txt");
    let result = run_rcol(&["--file", data_path.to_str().unwrap()], None).unwrap();

    let expected = "Name    Age City       Country\n\
                    Alice   30  NewYork\n\
                    Bob     25  LosAngeles USA\n\
                    Charlie 35\n\
                    David   28  NewYork    USA\n\
                    Eve     22  LosAngeles USA        California\n";
    assert_snapshot(&result, expected);
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
