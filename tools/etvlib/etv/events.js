/* Saying "this changed" instead of calling whoever shows it — as in the
 * analyzer of Race Analysis (`analyzer/events.js`).
 *
 * A module that changes something says so — `emit("engine")` — and whoever
 * shows it has subscribed in its own `init()`. Imports only point at what a
 * module uses, never back at who has to redraw, so the import graph has no
 * ring (`tools/tests/test_etv_import_graph.py`). An event says what happened,
 * never what to do. Listeners run at once, in the order they subscribed.
 *
 * Nothing runs on import.
 */

//: Every event there is. `on` and `emit` refuse others: a misspelt name would
//: be an event nobody ever hears.
export const EVENTS = {
  engine: "the engine torque table changed",
  request: "the torque request changed",
  settings: "the settings of the calculation changed",
  result: "the ETV map changed — calculated, fixed, edited or discarded",
  step: "another step is shown",
};

const listeners = new Map();

function known(name){
  if (!Object.hasOwn(EVENTS, name)) throw new Error(`unknown event "${name}"`);
}

//: Call *fn* whenever *name* is emitted.
export function on(name, fn){
  known(name);
  if (!listeners.has(name)) listeners.set(name, []);
  listeners.get(name).push(fn);
}

export function emit(name, ...args){
  known(name);
  for (const fn of listeners.get(name) || []) fn(...args);
}
