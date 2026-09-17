// The numbers of an editable table (tools/etvlib/etv/tablemath.js): run with
// `node --test tools/tests/js/`, or through pytest (test_frontend.py).

import assert from "node:assert/strict";
import { test } from "node:test";

import * as tm from "../../etvlib/etv/tablemath.js";

const TABLE = { rpm: [4000, 8000], axis: [0, 50, 100], values: [[0, 50, 60], [0, 70, 100]] };

test("a number may come with a decimal comma, and nothing is not zero", () => {
  assert.equal(tm.parseNumber("12,5"), 12.5);
  assert.equal(tm.parseNumber(" -3.25 "), -3.25);
  for (const text of ["", "  ", "abc", "1x", null, undefined]) assert.ok(Number.isNaN(tm.parseNumber(text)), String(text));
});

test("numbers are shown without a tail", () => {
  assert.deepEqual([4000, 12.5, 0.1 + 0.2, -0, NaN].map(tm.formatNumber), ["4000", "12.5", "0.3", "0", ""]);
});

test("breakpoints are typed as a list: sorted, each once, or refused", () => {
  assert.deepEqual(tm.parseBreakpoints("50, 0;12.5  100, 50"), [0, 12.5, 50, 100]);
  assert.equal(tm.parseBreakpoints("0, ten"), null);
  assert.equal(tm.parseBreakpoints("  "), null);
  assert.equal(tm.breakpointsText([0, 12.5, 100]), "0, 12.5, 100");
});

test("a paste from a spreadsheet is rows of cells", () => {
  assert.deepEqual(tm.parsePaste("1\t2\r\n3\t4\r\n"), [["1", "2"], ["3", "4"]]);
  assert.deepEqual(tm.parsePaste("1;2\n3;4"), [["1", "2"], ["3", "4"]]);
  assert.deepEqual(tm.parsePaste("7"), [["7"]]);
});

test("a pasted block lands at the cell, header included, and stops at the edge", () => {
  // Grid coordinates: row 0 is the column breakpoints, column 0 the RPM.
  const { table, bad } = tm.pasteInto(TABLE, 1, 2, [["55", "66", "77"], ["7,5", "x", "9"], ["1", "2", "3"]]);
  assert.deepEqual(table.values, [[0, 55, 66], [0, 7.5, 100]]);      // 77 and the third row fall off; "x" stays
  assert.equal(bad, 1);
  assert.deepEqual(TABLE.values[0], [0, 50, 60]);                     // the table given is left alone

  const whole = tm.pasteInto(TABLE, 0, 0, [["RPM", "0", "40", "100"], ["5000", "1", "2", "3"]]);
  assert.deepEqual(whole.table.axis, [0, 40, 100]);
  assert.deepEqual(whole.table.rpm, [5000, 8000]);
  assert.deepEqual(whole.table.values[0], [1, 2, 3]);
  assert.equal(whole.bad, 0);                                         // the corner holds nothing and is skipped
});

test("rows and columns follow their breakpoints", () => {
  const mixed = { rpm: [8000, 4000], axis: [100, 0, 50], values: [[100, 0, 70], [60, 0, 50]] };
  assert.deepEqual(tm.sorted(mixed), TABLE);
});

test("a new row or column continues the last step and copies the last values", () => {
  const more = tm.addColumn(tm.addRow(TABLE));
  assert.deepEqual(more.rpm, [4000, 8000, 12000]);
  assert.deepEqual(more.axis, [0, 50, 100, 150]);
  assert.deepEqual(more.values[2], [0, 70, 100, 100]);
  assert.deepEqual(tm.addRow({ rpm: [4000], axis: [0], values: [[1]] }).rpm, [4000, 4001]);
});

test("a table keeps at least one row and one column", () => {
  assert.deepEqual(tm.removeRow(TABLE, 0).rpm, [8000]);
  assert.deepEqual(tm.removeColumn(TABLE, 1).values, [[0, 60], [0, 100]]);
  const one = { rpm: [4000], axis: [0], values: [[1]] };
  assert.equal(tm.removeRow(one, 0), one);
  assert.equal(tm.removeColumn(one, 0), one);
});

test("what cannot be calculated with is said as a sentence", () => {
  assert.equal(tm.problem(TABLE, "engine torque table"), "");
  assert.match(tm.problem({ ...TABLE, values: [[0, NaN, 60], [0, 70, 100]] }, "torque request"), /torque request has a cell that is empty/);
  assert.match(tm.problem({ ...TABLE, rpm: [4000, 4000] }, "engine torque table"), /RPM breakpoint 4000 twice/);
  assert.match(tm.problem({ ...TABLE, axis: [0, 50, 50] }, "engine torque table"), /column breakpoint 50 twice/);
  assert.deepEqual([tm.lowest(TABLE), tm.highest(TABLE)], [0, 100]);
});
