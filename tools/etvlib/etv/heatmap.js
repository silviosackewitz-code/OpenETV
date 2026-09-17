/* A table as a heatmap on the scope: RPM upwards, the other axis across, the
 * value as a colour of the ramp (`ramp.js`), with the ramp and its ends at
 * the right. Breakpoints are drawn evenly — they are the cells of a table,
 * not positions on a scale.
 *
 * A canvas cannot take `var()`: the roles are read at every draw.
 *
 * Nothing runs on import.
 */

import { themeColour } from "/common.js";
import { rampColour } from "./ramp.js";
import { formatNumber, highest, lowest } from "./tablemath.js";

//: About *count* of *list*'s indices, evenly apart, the first and the last among them.
function ticks(list, count){
  const n = list.length, step = Math.max(1, Math.ceil((n - 1) / Math.max(1, count - 1)));
  const at = [];
  for (let i = 0; i < n - 1; i += step) at.push(i);
  if (at.length && n - 1 - at.at(-1) < step / 2) at.pop();
  return [...at, n - 1];
}

export function drawMap(canvas, table, { unit = "" } = {}){
  const ratio = devicePixelRatio || 1, w = canvas.clientWidth, h = canvas.clientHeight;
  if (!w || !h) return;
  canvas.width = w * ratio; canvas.height = h * ratio;
  const g = canvas.getContext("2d");
  g.scale(ratio, ratio);
  g.fillStyle = themeColour("--scope"); g.fillRect(0, 0, w, h);

  const left = 52, right = 70, top = 10, bottom = 24;
  const rows = table.rpm.length, cols = table.axis.length;
  const cw = (w - left - right) / cols, ch = (h - top - bottom) / rows;
  const lo = lowest(table), hi = highest(table), span = hi - lo || 1;
  table.values.forEach((row, i) => row.forEach((v, j) => {
    g.fillStyle = rampColour((v - lo) / span);
    // The lowest RPM at the bottom, as in a plot. Overlap by a hair: no seams.
    g.fillRect(left + j * cw, top + (rows - 1 - i) * ch, cw + .5, ch + .5);
  }));

  g.font = `11px ${themeColour("--mono")}`; g.fillStyle = themeColour("--scope-ink-2");
  g.textAlign = "center"; g.textBaseline = "top";
  for (const j of ticks(table.axis, Math.floor((w - left - right) / 48)))
    g.fillText(formatNumber(table.axis[j]), left + (j + .5) * cw, h - bottom + 5);
  g.textAlign = "right"; g.textBaseline = "middle";
  for (const i of ticks(table.rpm, Math.floor((h - top - bottom) / 22)))
    g.fillText(formatNumber(table.rpm[i]), left - 6, top + (rows - 1 - i + .5) * ch);

  // The ramp, with what its ends mean.
  const bx = w - right + 16, bh = h - top - bottom;
  for (let y = 0; y < bh; y++){ g.fillStyle = rampColour(1 - y / bh); g.fillRect(bx, top + y, 10, 1.5); }
  g.fillStyle = themeColour("--scope-ink-2"); g.textAlign = "left";
  g.textBaseline = "top"; g.fillText(`${formatNumber(hi)} ${unit}`.trim(), bx + 14, top);
  g.textBaseline = "bottom"; g.fillText(`${formatNumber(lo)} ${unit}`.trim(), bx + 14, top + bh);
}
