/* Step 5: the ETV map as a file — CSV, Excel, or a Mectronik DataSubset whose
 * paths and units default to those of the file the request came from.
 *
 * Nothing runs on import.
 */

import { $ } from "/common.js";
import { notify } from "/notify.js";
import { on } from "./events.js";
import { saveFile } from "./files.js";
import { state } from "./state.js";

const fields = () => [...document.querySelectorAll("[data-dss]")];

//: The defaults of the .dss fields: the app's, and over them what the
//: request's own .dss file said about its axes. What the user typed stays.
function defaults(){
  const meta = state.requestMeta ?? {};
  const from = { ...state.about?.dss_defaults, ...Object.fromEntries(
    ["rpm_path", "rpm_unit", "other_path", "other_unit"].filter(k => meta[k]).map(k => [k, meta[k]])) };
  for (const field of fields())
    if (!field.dataset.typed) field.value = from[field.dataset.dss] ?? "";
}

function show(){
  $("#exportBlank").hidden = Boolean(state.result);
  $("#exportBody").hidden = !state.result;
}

async function save(format){
  if (state.stale && !confirm("The tables or the settings changed since this map was calculated. Save it anyway?")) return;
  const dss = Object.fromEntries(fields().map(f => [f.dataset.dss, f.value.trim()]));
  try {
    const said = await saveFile({ table: state.result.table, format, name: "etv_map_throttle", dss });
    if (said) notify(said);
  } catch (error){
    notify(`The map was not saved: ${error.message}`, true);
  }
}

export function init(){
  const section = $('main > section[data-step="export"]');
  section.addEventListener("click", e => { const b = e.target.closest("[data-export]"); if (b && state.result) save(b.dataset.export); });
  section.addEventListener("input", e => { if (e.target.dataset.dss) e.target.dataset.typed = "1"; });
  on("result", show);
  on("request", defaults);
}

//: After `/api/about` arrived.
export const showDefaults = defaults;
