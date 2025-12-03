use clap::Parser;
use rcol::args::AppArgs;

#[test]
fn test_default_args() {
    let args = AppArgs::default();
    assert_eq!(args.sep, " ");
    assert_eq!(args.w, 1);
    assert_eq!(args.colsep, "│");
    assert!(!args.pp);
    assert!(!args.csv);
    assert!(args.columns.is_empty());
}

#[test]
fn test_parse_simple_flags() {
    let args = AppArgs::try_parse_from(&["rcol", "--pp", "--csv"]).unwrap();
    assert!(args.pp);
    assert!(args.csv);
}

#[test]
fn test_parse_short_flags() {
    let args = AppArgs::try_parse_from(&["rcol", "-p", "-n"]).unwrap();
    assert!(args.pp);
    assert!(args.num);
}

#[test]
fn test_parse_args_with_file() {
    let args = AppArgs::try_parse_from(&["rcol", "--file", "test.txt"]).unwrap();
    assert_eq!(args.file, Some("test.txt".to_string()));
}

#[test]
fn test_parse_args_with_header() {
    let args = AppArgs::try_parse_from(&["rcol", "--header", "Col1 Col2"]).unwrap();
    assert_eq!(args.header, Some("Col1 Col2".to_string()));
}

#[test]
fn test_parse_args_with_separator() {
    let args = AppArgs::try_parse_from(&["rcol", "--sep", ","]).unwrap();
    assert_eq!(args.sep, ",");
}

#[test]
fn test_parse_args_with_columns() {
    let args = AppArgs::try_parse_from(&["rcol", "1", "2", "3"]).unwrap();
    assert_eq!(args.columns.len(), 3);
    assert_eq!(args.columns[0], "1");
}

#[test]
fn test_parse_args_width() {
    let args = AppArgs::try_parse_from(&["rcol", "-w", "3"]).unwrap();
    assert_eq!(args.w, 3);
}

#[test]
fn test_parse_args_sortcol() {
    let args = AppArgs::try_parse_from(&["rcol", "--sortcol", "2"]).unwrap();
    assert_eq!(args.sortcol, Some(2));
}

#[test]
fn test_parse_args_gcol() {
    let args = AppArgs::try_parse_from(&["rcol", "--gcol", "1", "--gcolval"]).unwrap();
    assert_eq!(args.gcol, Some(1));
    assert!(args.gcolval);
}

#[test]
fn test_parse_args_filter() {
    let args = AppArgs::try_parse_from(&["rcol", "--filter", "test.*"]).unwrap();
    assert_eq!(args.filter, Some("test.*".to_string()));
}
