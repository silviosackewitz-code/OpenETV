/* A step that is a table: the engine torque and the torque request. The grid,
 * its buttons, loading a file (chosen or dropped) and the heatmap below.
 *
 * `mountTableStep({step, key, role, what, corner, event})`: *step* is the
 * section's `data-step`, *key* where the table stands in `state`, *role* what
 * a file is offered as (`/api/parse`), *what* names the table in a sentence,
 * *event* is emitted after every change. Returns `{show(table, meta)}`.
 *
 * Nothing runs on import.
 */

import { $ } from "/common.js";
import { icon } from "/icons.js";
import { notify } from "/notify.js";
import { emit, on } from "./events.js";
import { pickTable, readFile } from "./files.js";
import { mountGrid } from "./grid.js";
import { drawMap } from "./heatmap.js";
import { state } from "./state.js";
import { addColumn, addRow, removeColumn, removeRow } from "./tablemath.js";

export function mountTableStep({ step, key, role, what, corner, event }){
  const section = $(`main > section[data-step="${step}"]`);
  const map = section.querySelector("[data-map]");

  //: Every change of the table goes through here.
  function changed(table, meta){
    state[key] = table;
    if (meta !== undefined) state[`${key}Meta`] = meta;
    if (state.result) state.stale = true;
    emit(event);
  }
  const grid = mountGrid(section.querySelector("[data-grid]"), { corner, onChange: t => changed(t), onNote: notify });
  const show = (table, meta = null) => { grid.show(table); changed(grid.table(), meta); };
  const paint = () => { if (!section.hidden && state[key]) drawMap(map, state[key], { unit: "Nm" }); };

  async function load(file){
    const read = await readFile(file, role);
    if (!read.ok) return notify(read.error, true);
    const entry = await pickTable($("#pick"), file.name, read.tables);
    if (!entry) return;
    show(entry.table, read.kind === "dss" ? entry : null);
    notify(`${what[0].toUpperCase()}${what.slice(1)}: ${entry.path} loaded.`);
  }

  const acts = section.querySelector("[data-table-acts]");
  acts.innerHTML = `<button class="btn" data-act="load">${icon("import")}Load file …</button>`
    + `<button class="btn" data-act="row">${icon("plus")}Row</button><button class="btn" data-act="column">${icon("plus")}Column</button>`
    + `<button class="btn ghost" data-act="no-row" title="Removes the row of the cell that was last edited">− Row</button>`
    + `<button class="btn ghost" data-act="no-column" title="Removes the column of the cell that was last edited">− Column</button>`;
  acts.addEventListener("click", e => {
    const act = e.target.closest("[data-act]")?.dataset.act, at = grid.focused(), table = grid.table();
    if (!act || !table && act !== "load") return;
    if (act === "load"){
      const pick = $("#filePick");
      pick.onchange = () => { if (pick.files[0]) load(pick.files[0]); pick.value = ""; };
      pick.click();
    }
    if (act === "row") show(addRow(table), state[`${key}Meta`]);
    if (act === "column") show(addColumn(table), state[`${key}Meta`]);
    if (act === "no-row") at?.r > 0 ? show(removeRow(table, at.r - 1), state[`${key}Meta`]) : notify("Click into the row to remove first.");
    if (act === "no-column") at?.c > 0 ? show(removeColumn(table, at.c - 1), state[`${key}Meta`]) : notify("Click into the column to remove first.");
  });

  // A file dropped on the step is loaded into it.
  section.addEventListener("dragover", e => { e.preventDefault(); section.classList.add("drop"); });
  section.addEventListener("dragleave", () => section.classList.remove("drop"));
  section.addEventListener("drop", e => {
    e.preventDefault(); section.classList.remove("drop");
    if (e.dataTransfer.files[0]) load(e.dataTransfer.files[0]);
  });

  on(event, paint);
  on("step", paint);
  addEventListener("resize", paint);
  return { show };
}
