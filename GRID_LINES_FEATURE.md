# Grid Lines Between Rows and Columns

## Feature Overview

When using the `--pp` (or `-p`) pretty-print flag, rcol now draws horizontal grid lines (separators) between all rows and columns, creating a full grid table layout.

## Changes Made

### Modified Files

1. **src/formatter.rs**
   - **print_header() function**: Updated to always print a separator after the header when `draw_borders` is true (not just when `draw_ts` is true)
   - **print_data_rows() function**: Added logic to print horizontal separators between all data rows when `draw_borders` is true

### Implementation Details

The grid lines use the box-drawing characters:
- Horizontal separator: `─` (horizontal line)
- Vertical separator: `│` (already present)
- Corners and junctions: `├`, `┤`, `┼`, `┌`, `┐`, `└`, `┘`

## Usage

### Before (Grid lines only around outer border + after header)
```
┌─────────┬─────┬────────────┐
│ Name    │ Age │ City       │
│ Alice   │  30 │ NewYork    │
│ Bob     │  25 │ LosAngeles │
│ Charlie │  35 │ Chicago    │
│ David   │  28 │ NewYork    │
│ Eve     │  22 │ LosAngeles │
└─────────┴─────┴────────────┘
```

### After (Full grid with lines between all rows)
```
┌─────────┬─────┬────────────┐
│ Name    │ Age │ City       │
├─────────┼─────┼────────────┤
│ Alice   │  30 │ NewYork    │
├─────────┼─────┼────────────┤
│ Bob     │  25 │ LosAngeles │
├─────────┼─────┼────────────┤
│ Charlie │  35 │ Chicago    │
├─────────┼─────┼────────────┤
│ David   │  28 │ NewYork    │
├─────────┼─────┼────────────┤
│ Eve     │  22 │ LosAngeles │
└─────────┴─────┴────────────┘
```

## Command Line Usage

```bash
# Use --pp flag
rcol --pp < data.txt

# Use short -p flag
rcol -p < data.txt

# Combine with other options
rcol --pp --ts < data.txt
rcol -p -mb < data.txt
```

## Compatibility with Other Features

✅ **Multiline cells** - Grid lines work with cells containing `\n` markers
✅ **Quoted strings** - Grid lines work with quoted content spanning multiple lines
✅ **Column/row grouping** - Grid lines separate all rows including group divisions
✅ **Sorting** - Grid lines maintain separation after sorting
✅ **Custom separators** - Grid lines apply only when using box-drawing mode (`--pp` or `-p`)

## Examples

### Example 1: Simple Table
```
rcol --pp < data.txt
```

### Example 2: Multiline Content
```
cat <<EOF | rcol --pp
Name Age City
Alice 30 New\nYork
Bob 25 Los\nAngeles
