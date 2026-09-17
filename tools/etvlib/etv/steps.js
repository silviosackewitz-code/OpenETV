/* The sidebar: which step is shown, and one line per step on how it stands.
 *
 * Nothing runs on import.
 */

import { $ } from "/common.js";
import { emit, on } from "./events.js";
import { state } from "./state.js";
import { formatNumber, highest } from "./tablemath.js";

const tabs = () => [...document.querySelectorAll("#steps [data-step]")];

//: Show *step*.
export function go(step){
  state.step = step;
  for (const tab of tabs()) tab.setAttribute("aria-selected", String(tab.dataset.step === step));
  for (const section of document.querySelectorAll("main > section"))
    section.hidden = section.dataset.step !== step;
  emit("step", step);
}

const size = t => t ? `${t.rpm.length} × ${t.axis.length}, up to ${formatNumber(highest(t))} Nm` : "no table";

function lines(){
  const say = (step, text, warn = false) => {
    const small = $(`#steps [data-step="${step}"] small`);
    small.textContent = text; small.classList.toggle("warn", warn);
  };
  say("engine", size(state.engine));
  say("request", size(state.request));
  const { threshold, tolerance } = state.settings;
  say("calculation", threshold == null ? "" : `above ${formatNumber(threshold)} rpm the largest throttle, ± ${formatNumber(tolerance)} Nm`);
  const r = state.result;
  say("result", !r ? "not calculated" : state.stale ? "out of date — calculate again"
    : r.counts.saturated ? `${r.counts.saturated} of ${r.table.rpm.length * r.table.axis.length} cells saturated` : "calculated",
    Boolean(r && state.stale));
  say("export", r ? (state.stale ? "the map is out of date" : "CSV, Excel, .dss") : "nothing yet", Boolean(r && state.stale));
  for (const section of document.querySelectorAll('main > section[data-step="result"], main > section[data-step="export"]'))
    section.dataset.stale = r && state.stale ? "1" : "0";
}

export function init(){
  $("#steps").addEventListener("click", e => { const tab = e.target.closest("[data-step]"); if (tab) go(tab.dataset.step); });
  // Up and down move through the steps, as in a list of tabs.
  $("#steps").addEventListener("keydown", e => {
    if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
    e.preventDefault();
    const all = tabs(), i = all.findIndex(t => t.dataset.step === state.step) + (e.key === "ArrowDown" ? 1 : -1);
    const tab = all[Math.max(0, Math.min(all.length - 1, i))];
    go(tab.dataset.step); tab.focus();
  });
  for (const name of ["engine", "request", "settings", "result"]) on(name, lines);
}
