use clap::Parser;

/// rcol - Rust Column Formatter
///
/// Format and shape unformatted ASCII text into columns.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct AppArgs {
    /// Read input from FILENAME
    #[arg(short = 'f', long)]
    pub file: Option<String>,

    /// Define a custom header line
    #[arg(short = 'H', long)]
    pub header: Option<String>,

    /// Define the input separator
    #[arg(short = 's', long, default_value = " ")]
    pub sep: String,

    /// Treat multiple consecutive separators as a single delimiter
    #[arg(short = 'm', long)]
    pub mb: bool,

    /// Set padding width between columns
    #[arg(short = 'w', long, default_value_t = 1)]
    pub w: usize,

    /// Define the string used for column separation in non-pretty-print mode
    #[arg(short = 'C', long, default_value = "│")]
    pub colsep: String,

    /// Process only lines matching the given REGEX
    #[arg(short = 'F', long)]
    pub filter: Option<String>,

    /// Sort output by column N (1-based index)
    #[arg(short = 'S', long)]
    pub sortcol: Option<usize>,

    /// Group by column N
    #[arg(short = 'g', long)]
    pub gcol: Option<usize>,

    /// When using -gcol, keep the repeated values instead of replacing them with empty strings
    #[arg(long)]
    pub gcolval: bool,

    /// No Format: Do not align columns to a common width
    #[arg(long)]
    pub nf: bool,

    /// No Numerical: Disable automatic right-alignment of numerical values
    #[arg(long)]
    pub nn: bool,

    /// No Headline: Treat the first line of input as data, not a header
    #[arg(long)]
    pub nhl: bool,

    /// Title Separator: Draw a line between the header and data
    #[arg(long)]
    pub ts: bool,

    /// Footer Separator: Draw a line before the last row of data
    #[arg(long)]
    pub fs: bool,

    /// Column Separator: Draw a vertical line between columns
    #[arg(long)]
    pub cs: bool,

    /// Pretty Print: Draw a border around the table using Unicode box-drawing characters
    #[arg(short = 'p', long)]
    pub pp: bool,

    /// Remove Header: Discard the first line of input
    #[arg(long)]
    pub rh: bool,

    /// Numbering: Add a row with column numbers at the top
    #[arg(short = 'n', long)]
    pub num: bool,

    /// Output as CSV
    #[arg(long)]
    pub csv: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Output as YAML
    #[arg(long)]
    pub yaml: bool,

    /// Output as HTML
    #[arg(long)]
    pub html: bool,

    /// JSON Title Column: Use the first column as the key for JSON objects
    #[arg(long)]
    pub jtc: bool,

    /// Print parameter verification info
    #[arg(short = 'v', long)]
    pub verify: bool,

    /// Specify which columns to output
    #[arg(trailing_var_arg = true)]
    pub columns: Vec<String>,

    /// Output comprehensive man page
    #[arg(short = 'M', long)]
    pub manpage: bool,
}

impl Default for AppArgs {
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
            yaml: false,
            html: false,
            jtc: false,
            verify: false,
            columns: Vec::new(),
            manpage: false,
        }
    }
}