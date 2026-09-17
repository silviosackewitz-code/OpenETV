// Placeholder of the window (docs/plan.md, step 5): one round through the API
// with the sample tables, and the controls of the look — so that a start
// shows at once whether server, key, calculation and theme work together.

import { $, getJson, postJson } from "/common.js";
import { followLook, mountAppearance } from "/theme.js";

async function round() {
  const about = await getJson("/api/about");
  const sample = await getJson("/api/sample");
  if (!about.ok || !sample.ok) throw new Error(about.error || sample.error);
  const result = await postJson("/api/calculate", { engine: sample.engine, request: sample.request });
  if (!result.ok) throw new Error(result.error);
  const { rpm, axis } = result.table;
  return `Version ${about.version}. The sample tables give an ETV map of ${rpm.length} × ${axis.length} ` +
    `cells, ${result.counts.saturated} of them saturated. The window itself is not built yet.`;
}

async function start() {
  mountAppearance($("#appearance"));
  followLook();
  const state = $("#state");
  try {
    state.textContent = await round();
  } catch (error) {
    state.textContent = `The server did not answer as expected: ${error.message}`;
    state.parentElement.classList.add("danger");
  }
}

start();
