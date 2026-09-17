/* Step 4: the ETV map — what was calculated, what happened in its cells, and
 * the fixes. The map can be edited by hand; a fix or an edit is part of the
 * result, a new calculation starts over.
 *
 * Nothing runs on import.
 */

import { $, esc, postJson } from "/common.js";
import { notify } from "/notify.js";
import { emit, on } from "./events.js";
import { mountGrid } from "./grid.js";
import { drawMap } from "./heatmap.js";
import { state } from "./state.js";
import { formatNumber } from "./tablemath.js";

let grid = null;

function notes(){
  const r = state.result, list = [];
  if (r && state.stale) list.push(["danger", "The tables or the settings changed since this map was calculated. Calculate again before using it."]);
  if (r?.outside.length) list.push(["danger", `${r.outside.map(formatNumber).join(", ")} rpm lie outside the engine torque table. `
    + "Its first or last RPM row was used there — what the engine really gives at these RPM is not known."]);
  if (r?.counts.saturated) list.push(["", `${r.counts.saturated} cell(s) are saturated: the request reaches what the engine has at that RPM, within the tolerance.`]);
  if (r?.counts.below_min) list.push(["", `${r.counts.below_min} cell(s) ask for less than the engine gives at closed throttle; the throttle is 0 % there.`]);
  if (r?.counts.non_monotonic) list.push(["danger", `${r.counts.non_monotonic} cell(s): the engine's torque sags after its peak and falls below the request `
    + "again at a larger throttle. The smallest throttle that delivers the torque was used — check these cells."]);
  $("#resultNotes").innerHTML = list.map(([kind, text]) => `<p class="banner ${kind}"><span class="grow">${esc(text)}</span></p>`).join("");
}

function paint(){
  const r = state.result, section = $('main > section[data-step="result"]');
  $("#resultBlank").hidden = Boolean(r);
  $("#resultLegend").hidden = !r;
  $("#resultMap").hidden = !r;
  for (const button of section.querySelectorAll("[data-fix]")) button.disabled = !r;
  notes();
  if (r && !section.hidden) drawMap($("#resultMap"), r.table, { unit: "%" });
}

//: The whole result changed: a calculation or a fix.
function show(){
  grid.show(state.result?.table ?? null);
  paint();
}

async function fix(kind){
  const body = { table: state.result.table, fix: kind, ramp_to: +$("#fixRamp").value };
  const fixed = await postJson("/api/postprocess", body);
  if (!fixed.ok) return notify(fixed.error, true);
  state.result.table = fixed.table;
  state.settings.rampTo = body.ramp_to;
  emit("result");
  notify(fixed.changed ? `${fixed.changed} cell(s) changed.` : "Nothing to change.");
}

export function init(){
  grid = mountGrid($("#resultGrid"), {
    corner: "RPM \\ Pedal %", onNote: notify,
    // The marks say what the calculation did; they stay through fixes and edits.
    mark: (i, j) => { const word = state.result?.status[i]?.[j]; return word && word !== "ok" ? word : ""; },
    // An edit by hand: the map is what the grid says, nothing else changed.
    onChange: table => { state.result.table = table; paint(); },
  });
  $('main > section[data-step="result"] .acts').addEventListener("click", e => {
    const button = e.target.closest("[data-fix]");
    if (button && state.result) fix(button.dataset.fix);
  });
  on("result", show);
  for (const name of ["engine", "request", "settings", "step"]) on(name, paint);
  addEventListener("resize", paint);
}
