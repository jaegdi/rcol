# rcol - Rust Column Formatter

## NAME
**rcol** - format and shape unformatted ASCII text into columns

## SYNOPSIS
**rcol** [*OPTIONS*] [*COLUMNS*]...

## DESCRIPTION
**rcol** reads text from standard input or a file, splits it into columns, and formats it into a justified table, CSV, JSON, or HTML-table. It is designed to turn unreadable, space-separated output (eg. like 
- `oc get ...` over several namespaces - 
- or `ls -l` over different dirs or recursive over dirs 
- or `ps aux`
  
) into structured, readable data.

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
    **Filename**. Read **input** from *FILENAME*. If stdin is also provided, they are combined.

*   `-sep='CHAR'`
    **Seperator**. Define the **input** separator (default is space `' '`).

*   `-mb`
    **More Blanks**. Treat multiple consecutive separators as a single delimiter. Useful for aligning pre-formatted text.

*   `-header='HEADER'`
    **Header**. Define a custom header line. Headers must be separated by the same separator as the **input** data.
    These headers are applied to the **output** columns.
    Headers starting with `-` are right-adjusted.

*   `-nhl`
    **No Headline**. Treat the first line of **input** as data, not a header.

*   `-rh`
    **Remove Header**. Discard the first line of **input**.

### Formatting
*   `-w=N`
    **Padding Width**. Set padding width (number of spaces) between columns (default `1`).

*   `-pp`
    **Pretty Print**. Draw a border around the table using Unicode box-drawing characters.

*   `-ts`
    **Title Separator**. Draw a line between the header and data. Implied if `-header` is set.

*   `-fs`
    **Footer Separator**. Draw a line before the last row of data.

*   `-cs`
    **Column Separator**. Draw a vertical line between columns.

*   `-colsep='STR'`
    **Column Seperator**Define the string used for column separation in non-pretty-print mode (default `|`).

*   `-num`
    **Numbering**. Add a row with column numbers at the top.
    These numbers correspond to the **input** column indices.

*   `-nf`
    **No Format**. Do not align columns to a common width.

*   `-nn`
    **No Numerical**. Disable automatic right-alignment of numerical values.

### Processing
*   `-filter='REGEX'`
    **Filter**. Process only lines matching the given *REGEX*.

*   `-sortcol=N`
    **Sort**. Sort output by column *N* (1-based index).

*   `-gcol=N`
    **Grouping Column**. Group by column *N*. Replaces repeated values in this column with empty strings.

*   `-gcolval`
    **Grouping Column Value**. When using `-gcol`, keep the repeated values instead of replacing them with empty strings.

### Output Formats
*   `-csv`
    **CSV**. Output as Comma Separated Values.

*   `-json`
    **JSON**. Output as JSON.

*   `-jtc`
    **JSON Title Column**. Use the first column as the key for JSON objects (requires headers).

*   `-html`
    **HTML**. Output as an HTML table.

### General
*   `-help`, `-h`
    **Help**. Print help message.

*   `-man`
    **Manual**. Print manual.

*   `-v`, `-verify`
    **Verify**. Print parameter verification info.

## COLUMNS
Specify which columns to output using 1-based indices.
*   `1 2 3` : Select columns 1, 2, and 3.
*   `4 1 7 5` : Select columns 4, 1, 7, and 5. But did not output columns 2, 3 and 6.
*   `1:3` : Select columns 1 through 3.
*   `3:1` : Select columns 3, 2, and 1 (reverse order).

If no columns are specified, all columns are output.

## RUST Doc

[rcol rust doc](doc/doc/rcol/index.html)

## EXAMPLES

For the examples, simple commands like `ls` or `ps` are used as table providers to keep the reproducibility of the `rcol` examples simple. However, the actual purpose of `rcol` is not necessarily clear from these examples.
This becomes more apparent when formatting the output of `oc get pods -A` with `rcol`. Without using `rcol`, the column widths of the output vary per namespace.
For example: `oc get pods -A --no-headers | rcol -mb -pp` then formats the column widths across all namespaces. This makes the output appear as a neatly formatted table.

**1. Basic formatting of `ls -l` output:**
```bash
/usr/bin/ls -ln | rcol -mb -pp
```
Example Result:
```
┌────────────┬────┬──────┬──────┬───────┬────┬─────┬───────┬──────────────────┐
│ insgesamt  │ 36 │      │      │       │    │     │       │                  │
│ -rw-r--r-- │  1 │ 1000 │ 1000 │  4309 │ 1. │ Dez │ 17:28 │ Cargo.lock       │
│ -rw-r--r-- │  1 │ 1000 │ 1000 │   171 │ 1. │ Dez │ 17:28 │ Cargo.toml       │
│ -rw-r--r-- │  1 │ 1000 │ 1000 │ 10764 │ 2. │ Dez │ 12:14 │ README.md        │
│ drwxr-xr-x │  1 │ 1000 │ 1000 │    92 │ 1. │ Dez │ 12:25 │ src              │
│ drwxr-xr-x │  1 │ 1000 │ 1000 │    98 │ 2. │ Dez │ 09:06 │ target           │
│ -rw-r--r-- │  1 │ 1000 │ 1000 │   969 │ 1. │ Dez │ 13:02 │ test_data_02.txt │
│ -rw-r--r-- │  1 │ 1000 │ 1000 │  1839 │ 1. │ Dez │ 17:22 │ test_data_03.txt │
│ -rw-r--r-- │  1 │ 1000 │ 1000 │   103 │ 1. │ Dez │ 12:27 │ test_data.txt    │
└────────────┴────┴──────┴──────┴───────┴────┴─────┴───────┴──────────────────┘
```

**2. Select specific columns (Permissions, User, Name):**
```bash
/usr/bin/ls -ln | rcol -mb -filter=':' -header="PERM USER NAME" 1 3 9
```
Example Result:
```
RIGHTS       USER   NAME             
────────────────────────────────────
-rw-r--r--   1000   Cargo.lock       
-rw-r--r--   1000   Cargo.toml       
-rw-r--r--   1000   README.md        
drwxr-xr-x   1000   src              
drwxr-xr-x   1000   target           
-rw-r--r--   1000   test_data_02.txt 
-rw-r--r--   1000   test_data_03.txt 
-rw-r--r--   1000   test_data.txt    
```

**3. Sort by size (column 5) and show headers:**
```bash
/usr/bin/ls -ln  | rcol -mb -header="PERM LINKS USER GROUP SIZE DAY MONTH TIME NAME" -sortcol=5 -pp
```
Example Result:
```
┌────────────┬───────┬──────┬───────┬──────┬─────┬───────┬───────┬──────────────────┐
│ PERM       │ LINKS │ USER │ GROUP │ SIZE │ DAY │ MONTH │ TIME  │ NAME             │
├────────────┼───────┼──────┼───────┼──────┼─────┼───────┼───────┼──────────────────┤
│ insgesamt  │    32 │      │       │      │     │       │       │                  │
│ drwxr-xr-x │     1 │ 1000 │  1000 │   92 │  1. │ Dez   │ 12:25 │ src              │
│ drwxr-xr-x │     1 │ 1000 │  1000 │   98 │  2. │ Dez   │ 09:06 │ target           │
│ -rw-r--r-- │     1 │ 1000 │  1000 │  103 │  1. │ Dez   │ 12:27 │ test_data.txt    │
│ -rw-r--r-- │     1 │ 1000 │  1000 │  171 │  1. │ Dez   │ 17:28 │ Cargo.toml       │
│ -rw-r--r-- │     1 │ 1000 │  1000 │  969 │  1. │ Dez   │ 13:02 │ test_data_02.txt │
│ -rw-r--r-- │     1 │ 1000 │  1000 │ 1839 │  1. │ Dez   │ 17:22 │ test_data_03.txt │
│ -rw-r--r-- │     1 │ 1000 │  1000 │ 4309 │  1. │ Dez   │ 17:28 │ Cargo.lock       │
│ -rw-r--r-- │     1 │ 1000 │  1000 │ 4352 │  2. │ Dez   │ 10:18 │ README.md        │
└────────────┴───────┴──────┴───────┴──────┴─────┴───────┴───────┴──────────────────┘
```

**4. Group by User (column 3):**
```bash
/usr/bin/ps auxn | head -n 10 | sed -e 's/^ *//g' | rcol -mb -gcol=1 -pp 1:11
```
Example Result
```txt
┌──────┬─────┬──────┬──────┬───────┬───────┬─────┬──────┬───────┬──────┬────────────────────────────────┐
│ USER │ PID │ %CPU │ %MEM │ VSZ   │ RSS   │ TTY │ STAT │ START │ TIME │ COMMAND                        │
├──────┼─────┼──────┼──────┼───────┼───────┼─────┼──────┼───────┼──────┼────────────────────────────────┤
│    0 │   1 │  0.0 │  0.0 │ 26088 │ 15556 │ ?   │ Ss   │ 00:17 │ 0:03 │ /usr/lib/systemd/systemd       │
│      │   2 │  0.0 │  0.0 │     0 │     0 │ ?   │ S    │ 00:17 │ 0:00 │ [kthreadd]                     │
│      │   3 │  0.0 │  0.0 │     0 │     0 │ ?   │ S    │ 00:17 │ 0:00 │ [pool_workqueue_release]       │
│      │   4 │  0.0 │  0.0 │     0 │     0 │ ?   │ I<   │ 00:17 │ 0:00 │ [kworker/R-rcu_gp]             │
│      │   5 │  0.0 │  0.0 │     0 │     0 │ ?   │ I<   │ 00:17 │ 0:00 │ [kworker/R-sync_wq]            │
│      │   6 │  0.0 │  0.0 │     0 │     0 │ ?   │ I<   │ 00:17 │ 0:00 │ [kworker/R-kvfree_rcu_reclaim] │
│      │   7 │  0.0 │  0.0 │     0 │     0 │ ?   │ I<   │ 00:17 │ 0:00 │ [kworker/R-slub_flushwq]       │
│      │   8 │  0.0 │  0.0 │     0 │     0 │ ?   │ I<   │ 00:17 │ 0:00 │ [kworker/R-netns]              │
│      │  13 │  0.0 │  0.0 │     0 │     0 │ ?   │ I<   │ 00:17 │ 0:00 │ [kworker/R-mm_percpu_wq]       │
└──────┴─────┴──────┴──────┴───────┴───────┴─────┴──────┴───────┴──────┴────────────────────────────────┘
```

**5. Convert to JSON:**
```bash
 /usr/bin/ls -ln | rcol -mb -json -jtc -header="Name TIME MONTH DAY SIZE GROUP USER LINKS RIGHTS" 9:1 
```
Example Result:
```json
{
  "Cargo.toml": {
    "DAY": "1.",
    "GROUP": "1000",
    "LINKS": "1",
    "MONTH": "Dez",
    "RIGHTS": "-rw-r--r--",
    "SIZE": "171",
    "TIME": "17:28",
    "USER": "1000"
  },
  "README.md": {
    "DAY": "2.",
    "GROUP": "1000",
    "LINKS": "1",
    "MONTH": "Dez",
    "RIGHTS": "-rw-r--r--",
    "SIZE": "4352",
    "TIME": "10:18",
    "USER": "1000"
  },
  "src": {
    "DAY": "1.",
    "GROUP": "1000",
    "LINKS": "1",
    "MONTH": "Dez",
    "RIGHTS": "drwxr-xr-x",
    "SIZE": "92",
    "TIME": "12:25",
    "USER": "1000"
  },
  "target": {
    "DAY": "2.",
    "GROUP": "1000",
    "LINKS": "1",
    "MONTH": "Dez",
    "RIGHTS": "drwxr-xr-x",
    "SIZE": "98",
    "TIME": "09:06",
    "USER": "1000"
  }
}
```

**6. Filter for specific lines and format:**
```bash
 journalctl -ex | rcol -filter="ERROR" -cs 1:6
```
Example Result:
```txt
 Dez │ 02 │ 10:04:47 │ host │ vivaldi[46666]: │ [ERROR:chromium/ui/wayland/host/wayland_wp_color_manager.cc:273]                           
 Dez │ 02 │ 10:04:47 │ host │ vivaldi[46666]: │ [ERROR:chromium/ui/wayland/host/wayland_wp_color_manager.cc:191]                           
 Dez │ 02 │ 10:04:47 │ host │ vivaldi[46666]: │ [ERROR:chromium/ui/wayland/host/wayland_wp_color_manager.cc:273]                           
 Dez │ 02 │ 10:04:47 │ host │ vivaldi[46666]: │ [ERROR:chromium/ui/wayland/host/wayland_wp_color_manager.cc:191]                           
 Dez │ 02 │ 10:04:47 │ host │ vivaldi[46666]: │ [ERROR:chromium/ui/wayland/host/wayland_wp_color_management.cc:63]                 
 Dez │ 02 │ 10:04:48 │ host │ vivaldi[46666]: │ [ERROR:chromium/extensions/browser/service_worker_task_queue.cc:463]                       
 Dez │ 02 │ 10:04:50 │ host │ vivaldi[46666]: │ [ERROR:chromium/google_apis/gcm/engine/registration_request.cc:292]                                       
 Dez │ 02 │ 10:04:51 │ host │ vivaldi[46666]: │ [ERROR:chromium/services/network/restricted_cookie_manager.cc:1148]                                       
 Dez │ 02 │ 10:04:51 │ host │ vivaldi[46666]: │ [ERROR:chromium/services/network/restricted_cookie_manager.cc:1157]                       
```

**7. Supports complex formatting with Unicode icons, like the output of tools like `lsd` (strips containing ANSI codes for alignment):**
```bash
rcol -pp -mb -gcol=1 -sortcol=1 -nhl -header="RIGHTS USER GROUP SIZE UNIT DAY MONTH CAL TIME YEAR S NAME" -file=test_data_03.txt
```
Example Result:
```txt
┌────────────┬──────┬───────┬──────┬──────┬─────┬───────┬─────┬──────────┬──────┬───┬──────────────────┐
│ RIGHTS     │ USER │ GROUP │ SIZE │ UNIT │ DAY │ MONTH │ CAL │ TIME     │ YEAR │ S │ NAME             │
├────────────┼──────┼───────┼──────┼──────┼─────┼───────┼─────┼──────────┼──────┼───┼──────────────────┤
│ .rw-r--r-- │ dirk │ dirk  │   44 │ KB   │ Mon │ Nov   │  24 │ 12:00:40 │ 2025 │  │ Cargo.lock       │
│            │ dirk │ dirk  │  428 │ B    │ Mon │ Nov   │  24 │ 12:00:25 │ 2025 │  │ Cargo.toml       │
│            │ dirk │ dirk  │  395 │ B    │ Mon │ Nov   │  24 │ 11:00:12 │ 2025 │  │ config-lin.yaml  │
│            │ dirk │ dirk  │  411 │ B    │ Mon │ Nov   │  24 │ 11:00:12 │ 2025 │  │ config-mac.yaml  │
│            │ dirk │ dirk  │  404 │ B    │ Mon │ Nov   │  24 │ 11:00:12 │ 2025 │  │ config-win.yaml  │
│            │ dirk │ dirk  │  184 │ B    │ Mon │ Nov   │  24 │ 11:00:12 │ 2025 │  │ configy.yaml     │
│            │ dirk │ dirk  │   34 │ KB   │ Mon │ Nov   │  24 │ 11:00:12 │ 2025 │  │ LICENSE          │
│            │ dirk │ dirk  │  6.9 │ KB   │ Mon │ Nov   │  24 │ 14:51:56 │ 2025 │  │ README.md        │
│            │ dirk │ dirk  │   79 │ B    │ Mon │ Nov   │  24 │ 12:19:02 │ 2025 │  │ test_config.yaml │
│            │ dirk │ dirk  │  2.6 │ KB   │ Mon │ Nov   │  24 │ 11:00:12 │ 2025 │  │ testpass.kdbx    │
│            │ dirk │ dirk  │   10 │ B    │ Mon │ Nov   │  24 │ 11:00:12 │ 2025 │  │ testpasswd       │
│            │      │       │      │      │     │       │     │          │      │   │                  │
│ .rwxr-xr-x │ dirk │ dirk  │  1.5 │ KB   │ Mon │ Nov   │  24 │ 14:37:40 │ 2025 │  │ build.sh         │
│            │      │       │      │      │     │       │     │          │      │   │                  │
│ drwxr-xr-x │ dirk │ dirk  │  102 │ B    │ Mon │ Nov   │  24 │ 11:06:09 │ 2025 │ 󱧼 │ src              │
│            │ dirk │ dirk  │   80 │ B    │ Mon │ Nov   │  24 │ 11:48:36 │ 2025 │  │ target           │
└────────────┴──────┴───────┴──────┴──────┴─────┴───────┴─────┴──────────┴──────┴───┴──────────────────┘
```

## AUTHOR
Written by Dirk Jäger.
