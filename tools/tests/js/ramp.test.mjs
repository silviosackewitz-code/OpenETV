// The value ramp of the heatmaps (tools/etvlib/etv/ramp.js).

import assert from "node:assert/strict";
import { test } from "node:test";

import { darkInkOn, rampColour, rampRgb } from "../../etvlib/etv/ramp.js";

const luminance = ([r, g, b]) => 0.2126 * r + 0.7152 * g + 0.0722 * b;

test("the ramp runs from dark to light, so it reads without colour too", () => {
  const steps = Array.from({ length: 41 }, (_, i) => luminance(rampRgb(i / 40)));
  for (let i = 1; i < steps.length; i++) assert.ok(steps[i] > steps[i - 1], `step ${i}`);
  assert.ok(steps[0] < 40 && steps.at(-1) > 220);
});

test("outside 0…1, and for what is no number, the ends are held", () => {
  assert.deepEqual(rampRgb(-3), rampRgb(0));
  assert.deepEqual(rampRgb(7), rampRgb(1));
  assert.deepEqual(rampRgb(NaN), rampRgb(0));
  assert.match(rampColour(0.5), /^rgb\(\d+,\d+,\d+\)$/);
});

test("text on the ramp is dark where the ramp is light", () => {
  assert.equal(darkInkOn(0), false);
  assert.equal(darkInkOn(1), true);
});
