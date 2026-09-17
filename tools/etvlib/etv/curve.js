/* A curve on the scope: the shape of a torque request over the pedal.
 *
 * `drawCurve(canvas, xs, ys, labels)` — both axes run from 0 to 100. The
 * scope is dark in every theme; a canvas cannot take `var()`, so the roles
 * are read at every draw (`themeColour`), never kept.
 *
 * Nothing runs on import.
 */

import { themeColour } from "/common.js";

export function drawCurve(canvas, xs, ys, { x = "", y = "" } = {}){
  const ratio = devicePixelRatio || 1, w = canvas.clientWidth, h = canvas.clientHeight;
  canvas.width = w * ratio; canvas.height = h * ratio;
  const g = canvas.getContext("2d");
  g.scale(ratio, ratio);
  g.fillStyle = themeColour("--scope"); g.fillRect(0, 0, w, h);
  const left = 34, right = 10, top = 10, bottom = 24;
  const px = v => left + v / 100 * (w - left - right), py = v => h - bottom - v / 100 * (h - top - bottom);

  g.font = `11px ${themeColour("--mono")}`; g.lineWidth = 1;
  for (let v = 0; v <= 100; v += 25){
    g.strokeStyle = themeColour("--scope-grid");
    g.beginPath(); g.moveTo(px(v), py(0)); g.lineTo(px(v), py(100)); g.moveTo(px(0), py(v)); g.lineTo(px(100), py(v)); g.stroke();
    g.fillStyle = themeColour("--scope-ink-2");
    g.textAlign = "center"; g.textBaseline = "top"; g.fillText(v, px(v), py(0) + 5);
    g.textAlign = "right"; g.textBaseline = "middle"; g.fillText(v, left - 5, py(v));
  }
  g.fillStyle = themeColour("--scope-ink-3"); g.textAlign = "right"; g.textBaseline = "bottom";
  g.fillText(x, px(100), py(0) - 3);
  g.textAlign = "left"; g.textBaseline = "top"; g.fillText(y, px(0) + 4, py(100) + 2);

  g.strokeStyle = themeColour("--scope-mark"); g.lineWidth = 2; g.lineJoin = "round";
  g.beginPath();
  xs.forEach((v, i) => i ? g.lineTo(px(v), py(ys[i])) : g.moveTo(px(v), py(ys[i])));
  g.stroke();
}
