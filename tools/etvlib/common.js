/* Helpers shared by every page of the app.
 *
 * An ES module: each page imports what it uses
 * (`import { $, esc } from "/common.js"`; the analyzer's modules from
 * `"../common.js"`). Each helper lives here once, so an improvement reaches
 * every window instead of one of several copies. Nothing touches the page on
 * import, so Node can load it for tests.
 */

//: Shorthand for querySelector.
export const $ = s => document.querySelector(s);

//: Make text safe to put into HTML. Channel names, units and notes come from
//: files; a name like `<b>RPM` must show up as written, not as markup.
//: `null`/`undefined` become an empty string instead of "undefined".
export const esc = s => String(s ?? "").replace(/[&<>"']/g,
  m => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[m]));

//: The value of a colour role of theme.css, e.g. `themeColour("--scope")` — for
//: a canvas, which cannot take `var(--scope)`. Read at every draw rather than
//: kept: the theme can change while the window is open.
export const themeColour = name =>
  getComputedStyle(document.documentElement).getPropertyValue(name).trim();

//: Seconds as minutes and seconds with *decimals* places: (125.3, 1) → "2:05.3".
//: Rounds before splitting off the minutes — rounding the seconds afterwards
//: turned 59.96 into "0:60.0". Negative times get a minus sign, a missing
//: time ("NaN") shows as "–".
export function clock(s, decimals){
  if (!Number.isFinite(s)) return "–";
  const unit = 10 ** decimals, n = Math.round(Math.abs(s) * unit);
  const m = Math.floor(n / (60 * unit));
  const rest = ((n - m * 60 * unit) / unit).toFixed(decimals).padStart(decimals + 3, "0");
  return `${s < 0 && n ? "-" : ""}${m}:${rest}`;
}

//: Seconds as a lap time: 112.065 → "1:52.065".
export const fmtT = s => clock(s, 3);

//: Check a response before reading it. Without this, an error message from
//: the backend would be read as data — as Float32 samples it even draws
//: silently wrong curves.
export async function checkResponse(r){
  if (r.ok) return r;
  const z = await r.json().catch(() => ({}));
  const err = new Error(z.error || `HTTP ${r.status}`);
  // A machine-readable word for a frontend that must react to a specific
  // failure (compare.js's NO_TRACK_CODE) — sturdier than matching the start
  // of the message, which is free text and may change (compare.py's twin).
  if (z.code) err.code = z.code;
  throw err;
}
export const fetchJson = (url, o) => fetch(url, o).then(checkResponse).then(r => r.json());
export const fetchBinary = (url, o) => fetch(url, o).then(checkResponse).then(r => r.arrayBuffer());

//: POST *data* as JSON and read the answer. Never throws: a failure comes
//: back as {ok: false, error} with its reason — a message saying only
//: "could not …" leaves nothing to go on.
export const postJson = (url, data) => fetch(url, { method: "POST", body: JSON.stringify(data || {}) })
  .then(r => r.json().catch(() => ({ ok: false, error: `HTTP ${r.status} from ${url}` })))
  .catch(e => ({ ok: false, error: `no answer from the app: ${e.message}` }));

//: Read JSON and never throw: a failed request comes back as
//: {ok: false, error} with its reason, the same shape the backend sends for
//: a failure it saw itself. For a caller that shows the reason instead of
//: stopping — every window had written this one line out for itself.
export const getJson = (url, options) => fetchJson(url, options)
  .catch(e => ({ ok: false, error: e.message }));

//: The fastest of *laps* — but only among complete ones. An out or in lap
//: (`part`) is only a piece of a lap; with a 36 s in lap in an AiM export it
//: would otherwise be the "best time". Only if every lap is partial does one
//: of them get picked. `null` for no laps at all.
export function bestLap(laps){
  const complete = laps.filter(l => !l.part);
  const candidates = complete.length ? complete : laps;
  return candidates.length ? candidates.reduce((a, b) => b.dur < a.dur ? b : a) : null;
}
