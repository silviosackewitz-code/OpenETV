/* The shared state of the window. Everything else is private to its module.
 *
 * The server keeps nothing: the tables live here, and every request carries
 * what it needs. A table is `{rpm: […], axis: […], values: [[…], …]}`, rows
 * being RPM.
 */

export const state = {
  //: The step shown on the right: engine, request, calculation, result, export.
  step: "engine",
  engine: null,
  request: null,
  //: What a .dss file said about the request's axes — the defaults of a .dss export.
  requestMeta: null,
  //: The settings of the calculation; breakpoints as the user typed them.
  //: Empty breakpoints mean "those of the torque request".
  settings: { rpm: "", grip: "", threshold: null, tolerance: 0.3, rampTo: 20 },
  //: The last calculation: `{table, status, counts, outside}`; `null` before it.
  result: null,
  //: The inputs or settings changed after the result was calculated.
  stale: false,
  //: From `/api/about`: version, defaults of a .dss export, the generator's presets.
  about: null,
};
