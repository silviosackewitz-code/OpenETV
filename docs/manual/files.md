> Archived browser interface. For the native app, use the [current manual](../../rust/MANUAL.md).

# Files

Which files OpenETV reads and writes, and what a table has to look like.

## What can be loaded

| Ending | What it is |
| --- | --- |
| `.csv` | a table from a spreadsheet or another tool |
| `.xlsx`, `.xlsm` | Excel; the first sheet is read |
| `.dss` | Mectronik DataSubset, the ECU software's own format |

`.xls`, the old Excel format from before 2007, cannot be read. Open it once
and save it as `.xlsx` or CSV.

A file can also be dropped on the window; it is loaded into the step it is
dropped on.

## How a table has to look

The layout is the one a spreadsheet gives:

    RPM\Throttle[%],0,10,20,30
    3000,-6.7,9.6,21.8,31.0
    5000,-8.5,14.3,31.4,44.3

- The **first row** holds the column breakpoints, the **first column** the
  engine speeds. The corner cell may say anything; it is ignored.
- Rows and columns may be in any order — the app sorts them.
- A cell that is empty or not a number is refused, with the row and column it
  is in.
- Semicolons instead of commas, and a decimal comma, are understood as well —
  that is what a German spreadsheet exports.

## The `.dss` format

`.dss` is XML: one or more `table_3d` blocks, each with a value matrix and two
axis references, and the axes themselves as `vector` blocks with their
breakpoints and units.

- A file may hold **several tables**, for example one per gear. The app lists
  them and you pick one.
- RPM may be either axis; the app puts it in the rows either way.
- **The unit is read, not only shown.** Where the app needs torque, a table in
  `%` is refused with a sentence saying it is a throttle map. ECU exports are
  often named "demand" and hold throttle in % all the same.
- Writing and reading again gives exactly the same numbers; this is checked
  against real ECU exports.

## What the app writes

- The map as `.dss`, CSV or Excel — where the save dialog says.
- `~/.openetv/prefs.json` — theme, accent and whether the **?** marks are
  shown. Nothing else is written, and nothing leaves your machine.
