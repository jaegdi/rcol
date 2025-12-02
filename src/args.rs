use std::env;

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
    fn default() -> Self {
        Self {
            file: None,
            header: None,
            sep: " ".to_string(),
            mb: false,
            w: 1,
            colsep: "|".to_string(),
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
                        app_args.sortcol = Some(val_str.parse().map_err(|_| "Invalid value for -sortcol")?);
                    }
                    "gcol" => {
                        let val_str = parse_value(value, &args, &mut i)?;
                        app_args.gcol = Some(val_str.parse().map_err(|_| "Invalid value for -gcol")?);
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
