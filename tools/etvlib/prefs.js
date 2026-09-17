/* The view preferences every window shares: the help marks, the theme, the
 * accent (`/api/prefs`, kept in `library.json`).
 *
 * One place reads and writes them, whoever shows a preference subscribes:
 * `onPrefs(fn)` is called with all preferences after every read and every
 * save. A window reads them again when it gets the focus, so a change made in
 * one window reaches the others (`watchPrefs`, once per page).
 *
 * Nothing runs on import.
 */

import { fetchJson, postJson } from "./common.js";

const listeners = [];
//: Counts local changes: a read that started before one is out of date, and
//: so is the answer to a save that another save has overtaken.
let changes = 0;
let watching = false;

//: Call *fn(prefs)* whenever the preferences have been read or saved.
export function onPrefs(fn){ listeners.push(fn); }

const tell = prefs => { for (const fn of listeners) fn(prefs); };

//: Read the preferences and tell the listeners; returns them (null when the
//: backend did not answer).
export async function readPrefs(){
  const before = changes;
  const prefs = await fetchJson("/api/prefs").catch(() => null);
  if (prefs?.ok && before === changes) tell(prefs);
  return prefs?.ok ? prefs : null;
}

//: Change preferences in the backend (`{help_marks: false}`); the listeners
//: hear the answer, which holds all of them. Returns the answer.
export async function savePrefs(values){
  const mine = ++changes;
  const reply = await postJson("/api/prefs", values);
  if (reply.ok && mine === changes) tell(reply);
  return reply;
}

//: Read now and whenever the window comes to the front. Returns the first read.
export function watchPrefs(){
  if (!watching){
    watching = true;
    addEventListener("focus", readPrefs);
    document.addEventListener("visibilitychange", () => { if (!document.hidden) readPrefs(); });
  }
  return readPrefs();
}
