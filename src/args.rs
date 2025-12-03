use std::env;

/// Application arguments parsed from command-line input.
///
/// Contains all configuration options for the rcol application, including input/output
/// settings, formatting options, and column specifications.
#[derive(Debug, Clone)]
pub struct AppArgs {
    pub file: Option<String>,
    pub header: Option<String>,
    pub sep: String,
    pub mb: bool,
    pub w: usize,
    pub colsep: String,
    pub filter: Option<String>,
    pub sortcol: Option<usize>,
    pub gcol: Option<usize>,
    pub gcolval: bool,
    pub nf: bool,
    pub nn: bool,
    pub nhl: bool,
    pub ts: bool,
    pub fs: bool,
    pub cs: bool,
    pub pp: bool,
    pub rh: bool,
    pub num: bool,
    pub csv: bool,
    pub json: bool,
    pub html: bool,
    pub jtc: bool,
    pub help: bool,
    pub man: bool,
    pub verify: bool,
    pub columns: Vec<String>,
}

impl Default for AppArgs {
    /// Creates a new `AppArgs` instance with default values.
    ///
    /// Default values include:
    /// - Separator: single space
    /// - Column separator: Unicode pipe character (│)
    /// - Padding width: 1
    /// - All boolean flags: false
    fn default() -> Self {
        Self {
            file: None,
            header: None,
            sep: " ".to_string(),
            mb: false,
            w: 1,
            colsep: "│".to_string(),
            filter: None,
            sortcol: None,
            gcol: None,
            gcolval: false,
            nf: false,
            nn: false,
            nhl: false,
            ts: false,
            fs: false,
            cs: false,
            pp: false,
            rh: false,
            num: false,
            csv: false,
            json: false,
            html: false,
            jtc: false,
            help: false,
            man: false,
            verify: false,
            columns: Vec::new(),
        }
    }
}

/// Parses command-line arguments into an `AppArgs` structure.
///
/// Processes arguments from the command line, distinguishing between flags
/// (starting with '-') and column specifications. Named parameters must appear
/// before column numbers.
///
/// # Returns
///
/// - `Ok(AppArgs)` if arguments are successfully parsed
/// - `Err(String)` if an unknown flag or invalid value is encountered
///
/// # Errors
///
/// Returns an error if:
/// - An unknown flag is provided
/// - A flag requiring a value is missing its value
/// - A numeric parameter receives an invalid value
pub fn parse_args() -> Result<AppArgs, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut app_args = AppArgs::default();
    let mut i = 0;
    let mut parsing_flags = true;

    while i < args.len() {
        let arg = &args[i];

        if parsing_flags {
            if arg.starts_with('-') {
                // Handle flags
                let clean_arg = arg.trim_start_matches('-');
                // Split by '=' if present
                let (key, value) = if let Some(idx) = clean_arg.find('=') {
                    (&clean_arg[..idx], Some(&clean_arg[idx + 1..]))
                } else {
                    (clean_arg, None)
                };

                match key {
                    "file" => {
                        app_args.file = Some(parse_value(value, &args, &mut i)?);
                    }
                    "header" => {
                        app_args.header = Some(parse_value(value, &args, &mut i)?);
                    }
                    "sep" => {
                        app_args.sep = parse_value(value, &args, &mut i)?;
                    }
                    "mb" => app_args.mb = true,
                    "w" => {
                        let val_str = parse_value(value, &args, &mut i)?;
                        app_args.w = val_str.parse().map_err(|_| "Invalid value for -w")?;
                    }
                    "colsep" => {
                        app_args.colsep = parse_value(value, &args, &mut i)?;
                    }
                    "filter" => {
                        app_args.filter = Some(parse_value(value, &args, &mut i)?);
                    }
                    "sortcol" => {
                        let val_str = parse_value(value, &args, &mut i)?;
                        app_args.sortcol =
                            Some(val_str.parse().map_err(|_| "Invalid value for -sortcol")?);
                    }
                    "gcol" => {
                        let val_str = parse_value(value, &args, &mut i)?;
                        app_args.gcol =
                            Some(val_str.parse().map_err(|_| "Invalid value for -gcol")?);
                    }
                    "gcolval" => app_args.gcolval = true,
                    "nf" => app_args.nf = true,
                    "nn" => app_args.nn = true,
                    "nhl" => app_args.nhl = true,
                    "ts" => app_args.ts = true,
                    "fs" => app_args.fs = true,
                    "cs" => app_args.cs = true,
                    "pp" => app_args.pp = true,
                    "rh" => app_args.rh = true,
                    "num" => app_args.num = true,
                    "csv" => app_args.csv = true,
                    "json" => app_args.json = true,
                    "html" => app_args.html = true,
                    "jtc" => app_args.jtc = true,
                    "help" | "h" => app_args.help = true,
                    "man" => app_args.man = true,
                    "v" | "verify" => app_args.verify = true,
                    _ => {
                        // Unknown flag, assume it's a column number if it looks like one,
                        // but requirements say named params must be before column numbers.
                        // However, user might make mistakes.
                        // Strict interpretation: Unknown flag is an error or start of columns if it doesn't look like a flag?
                        // But wait, "All named parameters must be defined before the column numbers".
                        // So if we encounter something that is not a known flag, is it a column number?
                        // Column numbers don't start with '-'.
                        // So if it starts with '-', it's an unknown flag.
                        return Err(format!("Unknown flag: {}", arg));
                    }
                }
            } else {
                // Not starting with '-', must be start of columns
                parsing_flags = false;
                app_args.columns.push(arg.clone());
            }
        } else {
            // No longer parsing flags, everything else is a column spec
            app_args.columns.push(arg.clone());
        }
        i += 1;
    }

    Ok(app_args)
}

/// Extracts a value for a command-line flag.
///
/// If the value is provided inline (e.g., `-flag=value`), returns it directly.
/// Otherwise, looks ahead to the next argument position.
///
/// # Arguments
///
/// * `value` - Optional inline value from the flag
/// * `args` - Slice of all command-line arguments
/// * `i` - Current position in the arguments array (mutable, will be incremented if looking ahead)
///
/// # Returns
///
/// - `Ok(String)` containing the parsed value
/// - `Err(String)` if no value is available
fn parse_value(value: Option<&str>, args: &[String], i: &mut usize) -> Result<String, String> {
    if let Some(v) = value {
        Ok(v.to_string())
    } else {
        // Look ahead
        if *i + 1 < args.len() {
            *i += 1;
            Ok(args[*i].clone())
        } else {
            Err("Missing value for flag".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_parse_value_inline() {
        let args_vec = vec!["arg1".to_string(), "arg2".to_string()];
        let mut i = 0;

        let result = parse_value(Some("test_value"), &args_vec, &mut i);
        assert_eq!(result.unwrap(), "test_value");
        assert_eq!(i, 0); // Should not increment
    }

    #[test]
    fn test_parse_value_next_arg() {
        let args_vec = vec!["arg1".to_string(), "next_value".to_string()];
        let mut i = 0;

        let result = parse_value(None, &args_vec, &mut i);
        assert_eq!(result.unwrap(), "next_value");
        assert_eq!(i, 1); // Should increment
    }

    #[test]
    fn test_parse_value_missing() {
        let args_vec = vec!["arg1".to_string()];
        let mut i = 0;

        let result = parse_value(None, &args_vec, &mut i);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Missing value for flag");
    }

    #[test]
    fn test_parse_args_with_file() {
        // Create a test by manually constructing AppArgs
        let mut args = AppArgs::default();
        args.file = Some("test.txt".to_string());

        assert_eq!(args.file, Some("test.txt".to_string()));
    }

    #[test]
    fn test_parse_args_with_header() {
        let mut args = AppArgs::default();
        args.header = Some("Col1 Col2 Col3".to_string());

        assert_eq!(args.header, Some("Col1 Col2 Col3".to_string()));
    }

    #[test]
    fn test_parse_args_with_separator() {
        let mut args = AppArgs::default();
        args.sep = ",".to_string();

        assert_eq!(args.sep, ",");
    }

    #[test]
    fn test_parse_args_with_columns() {
        let mut args = AppArgs::default();
        args.columns = vec!["1".to_string(), "2".to_string(), "3".to_string()];

        assert_eq!(args.columns.len(), 3);
        assert_eq!(args.columns[0], "1");
    }

    #[test]
    fn test_parse_args_width() {
        let mut args = AppArgs::default();
        args.w = 3;

        assert_eq!(args.w, 3);
    }

    #[test]
    fn test_parse_args_sortcol() {
        let mut args = AppArgs::default();
        args.sortcol = Some(2);

        assert_eq!(args.sortcol, Some(2));
    }

    #[test]
    fn test_parse_args_gcol() {
        let mut args = AppArgs::default();
        args.gcol = Some(1);
        args.gcolval = true;

        assert_eq!(args.gcol, Some(1));
        assert!(args.gcolval);
    }

    #[test]
    fn test_parse_args_filter() {
        let mut args = AppArgs::default();
        args.filter = Some("test.*".to_string());

        assert_eq!(args.filter, Some("test.*".to_string()));
    }

    #[test]
    fn test_parse_args_all_boolean_flags() {
        let mut args = AppArgs::default();
        args.mb = true;
        args.nf = true;
        args.nn = true;
        args.nhl = true;
        args.ts = true;
        args.fs = true;
        args.cs = true;
        args.pp = true;
        args.rh = true;
        args.num = true;
        args.csv = true;
        args.json = true;
        args.html = true;
        args.jtc = true;

        assert!(args.mb);
        assert!(args.nf);
        assert!(args.nn);
        assert!(args.nhl);
        assert!(args.ts);
        assert!(args.fs);
        assert!(args.cs);
        assert!(args.pp);
        assert!(args.rh);
        assert!(args.num);
        assert!(args.csv);
        assert!(args.json);
        assert!(args.html);
        assert!(args.jtc);
    }
}
