/* An editable table: RPM down the side, breakpoints across the top, a number
 * in every cell. Takes a paste from a spreadsheet.
 *
 * `mountGrid(el, options)` builds it into *el* and returns
 * `{show(table), table(), focused()}`. The numbers themselves — parsing,
 * pasting, sorting — are `tablemath.js`; this module is the cells.
 *
 * Options: `corner` (the label of the corner cell), `onChange(table)` (called
 * after every edit that gave a number), `onNote(text, isError)`, and for a
 * table that is a result, `mark(row, col)` → a word that becomes
 * `data-mark` on the cell.
 *
 * Nothing runs on import.
 */

import { esc } from "/common.js";
import { formatNumber, parseNumber, parsePaste, pasteInto, sorted } from "./tablemath.js";

export function mountGrid(el, { corner = "", onChange = () => {}, onNote = () => {}, mark = null } = {}){
  let table = null;
  //: The cell that last had the focus, in grid coordinates (0 is the header).
  let last = null;

  function render(){
    if (!table){ el.innerHTML = ""; return; }
    const cell = (tag, r, c, v, extra = "") =>
      `<${tag} contenteditable="plaintext-only" spellcheck="false" inputmode="decimal" data-r="${r}" data-c="${c}"${extra}>`
      + `${esc(formatNumber(v))}</${tag}>`;
    el.innerHTML = `<table class="grid"><thead><tr><th class="corner">${esc(corner)}</th>${
      table.axis.map((v, j) => cell("th", 0, j + 1, v)).join("")}</tr></thead><tbody>${
      table.rpm.map((rpm, i) => `<tr>${cell("th", i + 1, 0, rpm)}${
        table.values[i].map((v, j) => {
          const word = mark?.(i, j);
          return cell("td", i + 1, j + 1, v, word ? ` data-mark="${esc(word)}"` : "");
        }).join("")}</tr>`).join("")}</tbody></table>`;
  }

  const at = (r, c) => el.querySelector(`[data-r="${r}"][data-c="${c}"]`);

  /* Take the cell's text into the table. A value is taken while it is typed
   * (`live`), so that what is on the screen is what is calculated with —
   * whatever happens to the focus afterwards. A breakpoint waits until its
   * cell is left: it moves its row or column, and moving it under the caret
   * would take the caret away. */
  function commit(cell, live = false){
    const r = +cell.dataset.r, c = +cell.dataset.c, v = parseNumber(cell.textContent);
    if (live && (r === 0 || c === 0)) return;
    const old = r === 0 ? table.axis[c - 1] : c === 0 ? table.rpm[r - 1] : table.values[r - 1][c - 1];
    cell.classList.toggle("bad", !Number.isFinite(v));
    if (!Number.isFinite(v) || v === old) return;
    if (r === 0) table.axis[c - 1] = v; else if (c === 0) table.rpm[r - 1] = v; else table.values[r - 1][c - 1] = v;
    // A breakpoint moved: its row or column goes where it belongs.
    if (r === 0 || c === 0){ table = sorted(table); render(); }
    onChange(table);
  }

  el.addEventListener("focusin", e => {
    const cell = e.target.closest("[data-r]");
    if (!cell) return;
    last = { r: +cell.dataset.r, c: +cell.dataset.c };
    // Typing replaces the number, as in a spreadsheet. Not at once: a click
    // gives the focus first and places its caret afterwards, which would undo
    // the selection — and "120" typed into 55 became 12055. A second click
    // into the same cell gives no focus and places the caret as usual.
    setTimeout(() => { if (document.activeElement === cell) getSelection().selectAllChildren(cell); }, 0);
  });
  el.addEventListener("input", e => { const cell = e.target.closest("[data-r]"); if (cell && table) commit(cell, true); });
  el.addEventListener("focusout", e => { const cell = e.target.closest("[data-r]"); if (cell && table) commit(cell); });
  el.addEventListener("keydown", e => {
    const cell = e.target.closest("[data-r]");
    if (!cell) return;
    if (e.key === "Enter"){
      e.preventDefault();
      const r = +cell.dataset.r + (e.shiftKey ? -1 : 1), c = +cell.dataset.c;
      cell.blur();
      at(r, c)?.focus();
    }
    if (e.key === "Escape"){
      const r = +cell.dataset.r, c = +cell.dataset.c;
      cell.textContent = formatNumber(r === 0 ? table.axis[c - 1] : c === 0 ? table.rpm[r - 1] : table.values[r - 1][c - 1]);
      cell.classList.remove("bad");
      cell.blur();
    }
  });
  el.addEventListener("paste", e => {
    const cell = e.target.closest("[data-r]");
    const block = parsePaste(e.clipboardData?.getData("text/plain"));
    // One cell is an ordinary paste into the cell.
    if (!cell || !table || (block.length === 1 && block[0].length === 1)) return;
    e.preventDefault();
    const pasted = pasteInto(table, +cell.dataset.r, +cell.dataset.c, block);
    table = sorted(pasted.table);
    render();
    onChange(table);
    if (pasted.bad) onNote(`${pasted.bad} pasted cell(s) were no numbers and stayed as they were.`, true);
  });

  return {
    show(t){ table = t ? { rpm: [...t.rpm], axis: [...t.axis], values: t.values.map(r => [...r]) } : null; last = null; render(); },
    table: () => table,
    focused: () => last,
  };
}
