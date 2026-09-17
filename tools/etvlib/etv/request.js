/* Step 2: the torque request — a table, and a generator that makes one from
 * the engine table: a shape over the pedal, in absolute torque or as a share
 * of the engine's maximum at each RPM. The curve is drawn by the server's
 * numbers (`/api/shape`), so there is one formula, not two.
 *
 * Nothing runs on import.
 */

import { $, postJson } from "/common.js";
import { notify } from "/notify.js";
import { drawCurve } from "./curve.js";
import { on } from "./events.js";
import { state } from "./state.js";
import { mountTableStep } from "./tablestep.js";
import { parseBreakpoints, problem } from "./tablemath.js";

let step = null, asked = 0;

function shape(){
  const kind = $("#genShape").value;
  return kind === "s_curve" ? { kind, center: +$("#genCenter").value, steepness: +$("#genSteep").value } : { kind };
}

async function preview(){
  for (const field of document.querySelectorAll("[data-s-curve]")) field.hidden = $("#genShape").value !== "s_curve";
  if (!$("#generator").open || $('main > section[data-step="request"]').hidden) return;
  const mine = ++asked;
  const curve = await postJson("/api/shape", { shape: shape(), max_percent: +$("#genMax").value });
  // A later question has been asked meanwhile: its answer is the one to draw.
  if (mine !== asked || !curve.ok) return;
  drawCurve($("#genCurve"), curve.grip, curve.percent, { x: "pedal %", y: "% of the engine's maximum" });
}

async function generate(){
  const wrong = !state.engine ? "There is no engine torque table to generate from." : problem(state.engine, "engine torque table");
  if (wrong) return notify(wrong, true);
  const grip = parseBreakpoints($("#genGrip").value);
  const rpm = $("#genRpm").value.trim() ? parseBreakpoints($("#genRpm").value) : state.engine.rpm;
  if (!grip || !rpm) return notify("The breakpoints have to be numbers, separated by commas.", true);
  const made = await postJson("/api/generate", { engine: state.engine, rpm, grip, shape: shape(),
    max_percent: +$("#genMax").value, per_rpm: document.querySelector('input[name="genScale"]:checked').value === "per_rpm" });
  if (!made.ok) return notify(made.error, true);
  step.show(made.table);
  notify("The torque request was generated.");
}

export function init(){
  step = mountTableStep({ step: "request", key: "request", role: "request", what: "torque request",
                          corner: "RPM \\ Pedal %", event: "request" });
  $("#generator").addEventListener("input", preview);
  $("#generator").addEventListener("toggle", preview);
  $("#genGo").addEventListener("click", generate);
  on("step", preview);
  addEventListener("resize", preview);
}

export const showRequest = table => step.show(table);
