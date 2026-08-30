# Multiline Cell Support

## Overview

The rcol application now supports multiline cell content with automatic row height synchronization. Linebreaks within cell values are signaled by the literal string `\n` (backslash followed by 'n').

## Features

### Automatic Row Height Synchronization

When a cell contains `\n`, the cell is split into multiple lines and displayed vertically. All cells in the same row are automatically aligned to the same height, with empty rows filled with spaces.

**Example:**
```
Input data (test_data_04.txt):
Name Age City
Alice 30 New\nYork
Bob 25 Los\nAngeles
Charlie 35 Chi\ncago

Output with pretty print (-p):
┌─────────┬─────┬───────────┐
│ Name    │ Age │ City      │
│ Alice   │  30 │ New       │
│         │     │ York      │
│ Bob     │  25 │ Los       │
│         │     │ Angeles   │
│ Charlie │  35 │ Chi       │
│         │     │ cago      │
└─────────┴─────┴───────────┘
```

### Features Supported with Multiline

- **Column width calculation**: Multiline content is properly measured. Column width is the maximum width of any line within a cell.
- **Alignment**: Numeric values are still right-aligned, text is left-aligned. The alignment is applied to each line consistently.
- **Row synchronization**: All cells in a row are expanded to match the row's maximum height.
- **All output formats**:
  - ASCII/Unicode table (with or without borders)
  - CSV format (preserves `\n` as-is)
  - JSON format (escapes `\n`)
  - HTML format (preserves `\n` as-is)
  - YAML format (supported)

### Formatting Options Compatible with Multiline

All existing formatting options work with multiline content:

| Option | Effect |
|--------|--------|
| `-p, --pp` | Pretty print with borders |
| `--pp --ts` | Pretty print with title separator between header and data |
| `--pp --fs` | Pretty print with footer separator before last row |
| `--cs` | Column separators (vertical lines) |
| `-w N, --width N` | Custom padding width between columns |
| `-C SEP, --colsep SEP` | Custom column separator string |
| `--ts` | Draw title separator |
| `--fs` | Draw footer separator |
| `--num` | Add column numbering row |
| `-S N, --sortcol N` | Sort by column |
| `-g N, --gcol N` | Group by column |
| Column selection | Select and reorder columns (1-based indices) |

## Examples

### Basic multiline table
```bash
cat test_data_04.txt | rcol --pp
```

### With title separator
```bash
cat test_data_04.txt | rcol --pp --ts
```

### Column selection with multiline
```bash
cat test_data_04.txt | rcol --pp --ts 1 3
```

### With increased padding
```bash
cat test_data_04.txt | rcol --pp --ts -w 2
```

### JSON output
```bash
cat test_data_04.txt | rcol --json
```

### CSV output
```bash
cat test_data_04.txt | rcol --csv
```

## Technical Details

### Implementation

The multiline support is implemented in the formatter module:

1. **Cell splitting**: The `split_cell_lines()` function splits each cell's content on the `\n` marker.

2. **Width calculation**: The `calculate_widths()` function now considers all lines within a cell and uses the maximum width.

3. **Row rendering**: Both `print_header()` and `print_data_rows()` now:
   - Split cells into lines
   - Calculate the maximum height (maximum number of lines in any cell)
   - Render each line of the row separately
   - Fill empty lines with spaces to maintain proper alignment

### Alignment Behavior

- **Numeric values**: Only the first line of a numeric value is checked. If the complete cell value is numeric, all lines are right-aligned.
- **Text values**: All lines are left-aligned.
- **Empty lines**: When a cell has fewer lines than the row's max height, empty space-filled lines are added.

### Backward Compatibility

All existing features and test cases continue to work unchanged. Data without `\n` characters is processed exactly as before.

## Data Format

Input files should use literal `\n` (backslash-n, not actual newline characters) to mark line breaks:

```
Name Age City
Alice 30 New\nYork
Bob 25 Los\nAngeles
```

NOT:
```
Name Age City
Alice 30 New
York
Bob 25 Los
Angeles
```

## Testing

Test with the provided test data file:
```bash
./target/release/rcol --pp < test_data_04.txt
```

All 27 integration tests pass with multiline support enabled.
