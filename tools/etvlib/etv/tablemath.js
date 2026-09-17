/* What an editable table does to its numbers — no page, no server, so that
 * Node can test it (`tools/tests/js/tablemath.test.mjs`).
 *
 * A table is `{rpm: […], axis: […], values: [[…], …]}`, rows being RPM. Every
 * function returns a new table and leaves the one it got alone.
 */

//: "12,5" and " 12.5 " are 12.5; anything else that is not a number is NaN —
//: also "", which `Number` would read as 0.
export function parseNumber(text){
  const t = String(text ?? "").trim().replace(",", ".");
  return t === "" ? NaN : Number(t);
}

//: 4000 rather than 4000.0; at most three decimals, no trailing zeros.
export const formatNumber = v => Number.isFinite(v) ? String(Math.round(v * 1000) / 1000) : "";

//: "0, 2, 3.5" → [0, 2, 3.5], sorted, each once; `null` when a part is no number.
//: A decimal comma cannot be told from a separator here, so parts are split
//: at commas, semicolons and blanks, and decimals need a point.
export function parseBreakpoints(text){
  const parts = String(text ?? "").split(/[\s,;]+/).filter(Boolean).map(Number);
  if (!parts.length || parts.some(v => !Number.isFinite(v))) return null;
  return [...new Set(parts)].sort((a, b) => a - b);
}

export const breakpointsText = list => list.map(formatNumber).join(", ");

//: What a spreadsheet puts on the clipboard: rows of cells, tabs between them
//: (semicolons when there is no tab). The last line break is not a row.
export function parsePaste(text){
  const lines = String(text ?? "").replace(/\r\n?/g, "\n").replace(/\n+$/, "").split("\n");
  const sep = lines.some(l => l.includes("\t")) ? "\t" : ";";
  return lines.map(l => l.split(sep).map(c => c.trim()));
}

const copy = t => ({ rpm: [...t.rpm], axis: [...t.axis], values: t.values.map(r => [...r]) });

/* Put a pasted *block* into the grid with its top left cell at (*row*, *col*).
 * The grid counts the header too: row 0 is the column breakpoints, column 0
 * the RPM, (0, 0) the corner, which holds nothing. What reaches beyond the
 * table is dropped. Returns `{table, bad}`, *bad* counting cells that were no
 * numbers and stayed as they were. */
export function pasteInto(table, row, col, block){
  const out = copy(table);
  let bad = 0;
  block.forEach((cells, i) => cells.forEach((cell, j) => {
    const r = row + i, c = col + j;
    if (r > out.rpm.length || c > out.axis.length || (r === 0 && c === 0)) return;
    const v = parseNumber(cell);
    if (!Number.isFinite(v)){ bad++; return; }
    if (r === 0) out.axis[c - 1] = v;
    else if (c === 0) out.rpm[r - 1] = v;
    else out.values[r - 1][c - 1] = v;
  }));
  return { table: out, bad };
}

//: Rows and columns in ascending order of their breakpoints.
export function sorted(table){
  const rows = table.rpm.map((_, i) => i).sort((a, b) => table.rpm[a] - table.rpm[b]);
  const cols = table.axis.map((_, j) => j).sort((a, b) => table.axis[a] - table.axis[b]);
  return { rpm: rows.map(i => table.rpm[i]), axis: cols.map(j => table.axis[j]),
           values: rows.map(i => cols.map(j => table.values[i][j])) };
}

//: The next breakpoint after the last: one more of the last step.
const next = list => list.length > 1 ? 2 * list.at(-1) - list.at(-2) : (list[0] ?? 0) + 1;

//: A new last row or column, its values those of the one before it.
export function addRow(table){
  const out = copy(table);
  out.rpm.push(next(table.rpm));
  out.values.push([...(table.values.at(-1) ?? table.axis.map(() => 0))]);
  return out;
}
export function addColumn(table){
  const out = copy(table);
  out.axis.push(next(table.axis));
  out.values.forEach(r => r.push(r.at(-1) ?? 0));
  return out;
}

//: Without row or column *index*; a table keeps at least one of each.
export function removeRow(table, index){
  if (table.rpm.length < 2) return table;
  const out = copy(table);
  out.rpm.splice(index, 1); out.values.splice(index, 1);
  return out;
}
export function removeColumn(table, index){
  if (table.axis.length < 2) return table;
  const out = copy(table);
  out.axis.splice(index, 1); out.values.forEach(r => r.splice(index, 1));
  return out;
}

//: Why a table cannot be calculated with, as a sentence — or "".
export function problem(table, what){
  const all = [...table.rpm, ...table.axis, ...table.values.flat()];
  if (all.some(v => !Number.isFinite(v))) return `The ${what} has a cell that is empty or not a number.`;
  for (const [name, list] of [["RPM", table.rpm], ["column", table.axis]]){
    const twice = list.find((v, i) => list.indexOf(v) !== i);
    if (twice !== undefined) return `The ${what} has the ${name} breakpoint ${formatNumber(twice)} twice.`;
  }
  return "";
}

export const lowest = t => Math.min(...t.values.flat());
export const highest = t => Math.max(...t.values.flat());
