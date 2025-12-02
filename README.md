# rcol - Rust Column Formatter

## NAME
**rcol** - format and shape unformatted ASCII text into columns

## SYNOPSIS
**rcol** [*OPTIONS*] [*COLUMNS*]...

## DESCRIPTION
**rcol** reads text from standard input or a file, splits it into columns, and formats it into a justified table, CSV, JSON, or HTML. It is designed to turn unreadable, space-separated output (like `ls -l` or `ps aux`) into structured, readable data.

It supports:
*   **Filtering** lines with regex.
*   **Sorting** by specific columns.
*   **Grouping** data to avoid repetition.
*   **Selecting** and **reordering** columns.
*   **Formatting** with custom separators, padding, and borders (ASCII or Unicode).
*   **Outputting** as ASCII table, CSV, JSON, or HTML.

## OPTIONS

### Input/Output
*   `-file=FILENAME`
    Read input from *FILENAME*. If stdin is also provided, they are combined.

*   `-sep='CHAR'`
    Define the input separator (default is space `' '`).

*   `-mb`
    **More Blanks**. Treat multiple consecutive separators as a single delimiter. Useful for aligning pre-formatted text.

*   `-header='HEADER'`
    Define a custom header line. Headers must be separated by the same separator as the data.
    These headers are applied to the **output** columns.
    Headers starting with `-` are right-adjusted.

*   `-nhl`
    **No Headline**. Treat the first line of input as data, not a header.

*   `-rh`
    **Remove Header**. Discard the first line of input.

### Formatting
*   `-w=N`
    Set padding width (number of spaces) between columns (default `1`).

*   `-pp`
    **Pretty Print**. Draw a border around the table using Unicode box-drawing characters.

*   `-ts`
    **Title Separator**. Draw a line between the header and data. Implied if `-header` is set.

*   `-fs`
    **Footer Separator**. Draw a line before the last row of data.

*   `-cs`
    **Column Separator**. Draw a vertical line between columns.

*   `-colsep='STR'`
    Define the string used for column separation in non-pretty-print mode (default `|`).

*   `-num`
    **Numbering**. Add a row with column numbers at the top.
    These numbers correspond to the **input** column indices.

*   `-nf`
    **No Format**. Do not align columns to a common width.

*   `-nn`
    **No Numerical**. Disable automatic right-alignment of numerical values.

### Processing
*   `-filter='REGEX'`
    Process only lines matching the given *REGEX*.

*   `-sortcol=N`
    Sort output by column *N* (1-based index).

*   `-gcol=N`
    Group by column *N*. Replaces repeated values in this column with empty strings.

*   `-gcolval`
    When using `-gcol`, keep the repeated values instead of replacing them with empty strings.

### Output Formats
*   `-csv`
    Output as Comma Separated Values.

*   `-json`
    Output as JSON.

*   `-jtc`
    **JSON Title Column**. Use the first column as the key for JSON objects (requires headers).

*   `-html`
    Output as an HTML table.

### General
*   `-help`, `-h`
    Print help message.

*   `-man`
    Print manual.

*   `-v`, `-verify`
    Print parameter verification info.

## COLUMNS
Specify which columns to output using 1-based indices.
*   `1 2 3` : Select columns 1, 2, and 3.
*   `1:3` : Select columns 1 through 3.
*   `3:1` : Select columns 3, 2, and 1 (reverse order).

If no columns are specified, all columns are output.

## EXAMPLES

**1. Basic formatting of `ls -l` output:**
```bash
ls -l | rcol -mb -pp
```

**2. Select specific columns (Permissions, User, Name):**
```bash
ls -l | rcol -mb 1 3 9
```

**3. Sort by size (column 5) and show headers:**
```bash
ls -l | rcol -mb -header="PERM LINKS USER GROUP SIZE MONTH DAY TIME NAME" -sortcol=5 -pp
```

**4. Group by User (column 3):**
```bash
ps aux | rcol -mb -gcol=1 -pp
```

**5. Convert to JSON:**
```bash
ls -l | rcol -mb -json
```

**6. Filter for specific lines and format:**
```bash
cat log.txt | rcol -filter="ERROR" -pp
```

**7. Complex formatting with Unicode icons (strips ANSI codes for alignment):**
```bash
rcol -pp -mb -gcol=1 -sortcol=1 -nhl -header="RIGHTS USER GROUP SIZE UNIT DAY MONTH CAL TIME YEAR S NAME" -file=test_data_03.txt
```

## AUTHOR
Written by Antigravity.
