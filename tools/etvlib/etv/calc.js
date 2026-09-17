/* Step 3: the settings of the calculation, and the calculation itself.
 *
 * Nothing runs on import.
 */

import { $, postJson } from "/common.js";
import { notify } from "/notify.js";
import { emit, on } from "./events.js";
import { state } from "./state.js";
import { go } from "./steps.js";
import { formatNumber, parseBreakpoints, parseNumber, problem } from "./tablemath.js";

//: The RPM the threshold can be: the engine's rows that hold any torque.
function thresholds(){
  const engine = state.engine, select = $("#setThreshold");
  const rows = engine ? engine.rpm.filter((_, i) => engine.values[i].some(v => v !== 0)) : [];
  const all = rows.length ? rows : engine?.rpm ?? [];
  if (!all.includes(state.settings.threshold)) state.settings.threshold = all[Math.floor(all.length / 2)] ?? null;
  select.innerHTML = all.map(v => `<option value="${v}">${formatNumber(v)}</option>`).join("");
  select.value = String(state.settings.threshold);
}

function read(){
  const s = state.settings;
  s.rpm = $("#setRpm").value; s.grip = $("#setGrip").value;
  s.threshold = +$("#setThreshold").value;
  s.tolerance = parseNumber($("#setTolerance").value);
  if (state.result) state.stale = true;
  emit("settings");
}

function placeholders(){
  const r = state.request;
  $("#setRpm").placeholder = r ? r.rpm.map(formatNumber).join(", ") : "";
  $("#setGrip").placeholder = r ? r.axis.map(formatNumber).join(", ") : "";
}

async function calculate(){
  const { engine, request, settings } = state;
  const wrong = !engine || !request ? "Both tables are needed." : problem(engine, "engine torque table") || problem(request, "torque request");
  if (wrong) return notify(wrong, true);
  const body = { engine, request, rpm_threshold: settings.threshold, tolerance: settings.tolerance };
  for (const [key, text, what] of [["rpm", settings.rpm, "RPM"], ["grip", settings.grip, "pedal"]]){
    if (!text.trim()) continue;
    body[key] = parseBreakpoints(text);
    if (!body[key]) return notify(`The ${what} breakpoints have to be numbers, separated by commas.`, true);
  }
  $("#calcGo").disabled = true;
  const answer = await postJson("/api/calculate", body);
  $("#calcGo").disabled = false;
  if (!answer.ok) return notify(answer.error, true);
  state.result = { table: answer.table, status: answer.status, counts: answer.counts, outside: answer.outside };
  state.stale = false;
  emit("result");
  go("result");
}

export function init(){
  $('main > section[data-step="calculation"] .form').addEventListener("change", read);
  $("#calcGo").addEventListener("click", calculate);
  on("engine", () => { thresholds(); emit("settings"); });
  on("request", placeholders);
  const note = () => { $("#calcNote").textContent = state.result && state.stale
    ? "The tables or the settings changed since the map was calculated." : ""; };
  for (const name of ["engine", "request", "settings", "result"]) on(name, note);
}
