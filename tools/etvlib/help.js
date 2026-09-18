/* Help marks: small "?" at buttons and functions that open the manual there.
 *
 * Shared by the main window (`/help.js`) and the analyzer (`../help.js`).
 * An element names its place in the manual with
 * `data-help=<page>#<heading id>`, e.g. `templates#saving-the-view` (the ids
 * come from `racelib/manual.py`); `initHelp` puts a "?" button at it:
 *
 *   data-help-place="after"   (default) right after the element
 *   data-help-place="in"      inside it, at the end (before a `.cnt` count)
 *   data-help-place="corner"  after it, on its top right corner, taking no room
 *   data-help-label="…"       what "Help on …" names; else the element's text
 *
 * A button that is a mark itself (class `helpmark`, e.g. the "?" beside a
 * row name) only gets `data-help`. The checkbox (**Help** in the main
 * window, **Help marks** in the analyzer, next to its **Help** manual
 * button) shows or hides every mark: the page's root gets
 * `data-help-marks="on"`, and the stylesheet hides `.helpmark` otherwise —
 * also before the setting has been read, so marks never flash up for
 * someone who switched them off.
 *
 * The setting is a view preference shared by all windows (`prefs.js`): a
 * window reads it again when it gets the focus.
 *
 * Nothing runs on import.
 */

import { onPrefs, savePrefs, watchPrefs } from "./prefs.js";

let open = () => {};
let checkbox = null;

function label(el){
  return el.dataset.helpLabel || el.getAttribute("aria-label") || el.textContent.trim();
}

//: The "?" button for the element *el*.
function markFor(el){
  const mark = document.createElement("button");
  mark.type = "button";
  mark.className = "helpmark";
  mark.dataset.help = el.dataset.help;
  mark.textContent = "?";
  mark.setAttribute("aria-label", `Help on ${label(el)}`);
  mark.title = `Open the manual: ${label(el)}`;
  return mark;
}

//: Put a mark at every element with `data-help` that has none yet.
export function placeMarks(root = document){
  for (const el of root.querySelectorAll("[data-help]:not(.helpmark)")){
    if (el.helpMark) continue;
    const mark = el.helpMark = markFor(el);
    const place = el.dataset.helpPlace || "after";
    if (place === "in"){
      const count = el.querySelector(":scope > .cnt");
      if (count) count.before(mark); else el.append(mark);
    } else {
      if (place === "corner") mark.classList.add("corner");
      el.after(mark);
    }
  }
}

//: Show or hide all marks and tick the checkbox to match.
export function showMarks(on){
  document.documentElement.dataset.helpMarks = on ? "on" : "off";
  if (checkbox) checkbox.checked = on;
}

/* Wire the marks of the page.
 *
 * *openManual(target)* opens the manual at "<page>#<heading>"; *box* is the
 * page's Help checkbox. Returns the preferences read at start. */
export function initHelp({ openManual, box }){
  open = openManual;
  checkbox = box;
  placeMarks();
  // In the capture phase: the mark sits in headers that are drag handles and
  // in <summary> elements that would open or close their section.
  document.addEventListener("click", e => {
    const mark = e.target.closest?.("button.helpmark[data-help]");
    if (!mark) return;
    e.preventDefault();
    e.stopPropagation();
    open(mark.dataset.help);
  }, true);
  box.addEventListener("change", async () => {
    showMarks(box.checked);
    const reply = await savePrefs({ help_marks: box.checked });
    if (!reply.ok) console.warn("could not keep the Help setting:", reply.error);
  });
  onPrefs(prefs => showMarks(prefs.help_marks));
  return watchPrefs();
}
