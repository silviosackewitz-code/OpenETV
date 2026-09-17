/* The value ramp of the heatmaps: a number as a colour.
 *
 * These colours are **data**, not look — they encode a value, as the ramp of
 * the track view does in Race Analysis — so they stand here and not in
 * theme.css; `test_look.py` knows this module by name as the one exception.
 * They are drawn on the scope, which is dark in every theme, so one ramp is
 * enough. It runs from dark to light, so that it still reads without colour.
 *
 * No page and no server: Node tests it (`tools/tests/js/ramp.test.mjs`).
 */

//: Stops from the lowest value to the highest, as [red, green, blue].
const STOPS = [[24, 28, 84], [35, 84, 160], [28, 140, 150], [70, 180, 100], [200, 215, 70], [253, 240, 150]];

//: The colour of *t*, 0 (lowest) to 1 (highest), as [r, g, b]; held at the ends.
export function rampRgb(t){
  const x = Math.min(1, Math.max(0, Number.isFinite(t) ? t : 0)) * (STOPS.length - 1);
  const i = Math.min(STOPS.length - 2, Math.floor(x)), f = x - i;
  return STOPS[i].map((c, k) => Math.round(c + (STOPS[i + 1][k] - c) * f));
}

export const rampColour = t => `rgb(${rampRgb(t).join(",")})`;

//: Whether text on the colour of *t* should be dark (true) or light.
export function darkInkOn(t){
  const [r, g, b] = rampRgb(t);
  return 0.2126 * r + 0.7152 * g + 0.0722 * b > 140;
}
