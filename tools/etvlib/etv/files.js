/* Files in and out: reading a table the user chose, saving one.
 *
 * Reading goes to `/api/parse`, which answers every table of the file; a
 * .dss can hold several, and the user picks (`pickTable`). Saving asks the
 * app for its save dialog (`/api/save`); in a browser there is none, and the
 * file is handed over as a download instead.
 *
 * Nothing runs on import.
 */

import { esc, postJson } from "/common.js";

const base64 = async blob => (await new Promise((done, fail) => {
  const reader = new FileReader();
  reader.onload = () => done(reader.result); reader.onerror = () => fail(reader.error);
  reader.readAsDataURL(blob);
})).split(",", 2)[1];

//: The tables of *file*, offered as *role* ("engine" or "request"):
//: `{ok, tables: [{path, table, refused, …}]}` or `{ok: false, error}`.
export async function readFile(file, role){
  return postJson("/api/parse", { name: file.name, data: await base64(file), role });
}

/* Let the user pick one of several tables in *dialog* (a `dialog.dialog` with
 * a `.body` and a close button). Tables that cannot serve say why and cannot
 * be picked. Resolves with the table's entry, or `null` when closed. */
export function pickTable(dialog, name, tables){
  const usable = tables.filter(t => !t.refused);
  if (tables.length === 1 && usable.length === 1) return Promise.resolve(usable[0]);
  dialog.querySelector("header b").textContent = `Tables in ${name}`;
  dialog.querySelector(".body").innerHTML = tables.map((t, i) => t.refused
    ? `<p class="refused"><b>${esc(t.path)}</b> — ${esc(t.refused)}</p>`
    : `<p><button type="button" class="btn" data-pick="${i}">${esc(t.path)}</button> `
      + `<span class="dim">${t.table.rpm.length} × ${t.table.axis.length}${t.unit ? ", " + esc(t.unit) : ""}</span></p>`).join("");
  return new Promise(done => {
    const finish = entry => { dialog.removeEventListener("close", closed); dialog.onclick = null; dialog.close(); done(entry); };
    const closed = () => finish(null);
    dialog.addEventListener("close", closed);
    dialog.onclick = e => {
      const pick = e.target.closest("[data-pick]");
      if (pick) finish(tables[+pick.dataset.pick]);
      if (e.target.closest("[data-close]")) finish(null);
    };
    dialog.showModal();
  });
}

//: Save what `/api/export` would answer for *body*. Resolves with a sentence
//: about what happened, `""` when the user cancelled; throws with the reason.
export async function saveFile(body){
  const saved = await postJson("/api/save", body);
  if (saved.ok) return saved.cancelled ? "" : `Saved ${saved.path}`;
  if (saved.code !== "no_window") throw new Error(saved.error);
  // No window of the app around the page: a browser. It saves downloads itself.
  const file = await postJson("/api/export", body);
  if (!file.ok) throw new Error(file.error);
  const bytes = Uint8Array.from(atob(file.data), c => c.charCodeAt(0));
  const link = Object.assign(document.createElement("a"),
    { href: URL.createObjectURL(new Blob([bytes], { type: file.mime })), download: file.name });
  document.body.append(link); link.click(); link.remove();
  setTimeout(() => URL.revokeObjectURL(link.href), 10000);
  return `${file.name} was handed to the browser's downloads`;
}
