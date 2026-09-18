/* Entry point of the window.
 *
 * Loaded as an ES module by etv.html. Every area is a module of its own and
 * nothing runs when a module is loaded; this file wires them up in a fixed
 * order and then fills the window with the sample tables, so that it is never
 * empty.
 */

import { $, getJson } from "/common.js";
import { placeIcons } from "/icons.js";
import { notify } from "/notify.js";
import { followLook, mountAppearance } from "/theme.js";
import { init as initCalc } from "./calc.js";
import { init as initEngine, showEngine } from "./engine.js";
import { init as initExporter, showDefaults } from "./exporter.js";
import { init as initHelp } from "./help.js";
import { init as initRequest, showRequest } from "./request.js";
import { init as initResult } from "./result.js";
import { state } from "./state.js";
import { go, init as initSteps } from "./steps.js";

function initLook(){
  const dialog = $("#look");
  mountAppearance($("#appearance"));
  $("#lookOpen").addEventListener("click", () => dialog.showModal());
  dialog.addEventListener("click", e => { if (e.target.closest("[data-close]")) dialog.close(); });
  followLook();
}

async function start(){
  placeIcons();
  initSteps();
  initEngine();
  initRequest();
  initCalc();
  initResult();
  initExporter();
  initHelp();
  initLook();

  const [about, sample] = await Promise.all([getJson("/api/about"), getJson("/api/sample")]);
  if (!about.ok || !sample.ok) return notify(`The app did not answer: ${about.error || sample.error}`, true);
  state.about = about;
  $("#version").textContent = about.version;
  showDefaults();
  showEngine(sample.engine);
  showRequest(sample.request);
  go("engine");
}

start();
