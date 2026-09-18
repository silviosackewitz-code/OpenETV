/* The manual in the window: the **Manual** button, the **?** marks at the
 * steps, and the dialog that shows a page.
 *
 * The marks themselves are `/help.js`, shared with Race Analysis: an element
 * says where it stands in the manual with `data-help="<page>#<heading>"`, and
 * a "?" is put at it while **Help** is ticked. This module says what happens
 * when one is clicked — the page opens in a dialog of this window, not in a
 * second window: there is only one.
 *
 * Nothing runs on import.
 */

import { $ } from "/common.js";
import { initHelp } from "/help.js";

//: Show the manual at *target* ("" the index, "etv-map#zero-gas-fix" a page).
export function openManual(target = ""){
  const dialog = $("#manual");
  const [page, heading] = String(target).split("#");
  // Always set the address, even for the same page: a heading has to be
  // jumped to again when the dialog is opened a second time.
  dialog.querySelector("iframe").src = `/manual/${page}${heading ? "#" + heading : ""}`;
  if (!dialog.open) dialog.showModal();
}

export function init(){
  const dialog = $("#manual");
  $("#manualOpen").addEventListener("click", () => openManual());
  dialog.addEventListener("click", e => { if (e.target.closest("[data-close]")) dialog.close(); });
  // The page inside the frame is ours; let it go when the dialog is closed.
  dialog.addEventListener("close", () => { dialog.querySelector("iframe").src = "about:blank"; });
  initHelp({ openManual, box: $("#helpMarks") });
}
