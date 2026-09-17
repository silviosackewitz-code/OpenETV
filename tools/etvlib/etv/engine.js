/* Step 1: the engine torque table. Nothing runs on import. */

import { mountTableStep } from "./tablestep.js";

let step = null;

export function init(){
  step = mountTableStep({ step: "engine", key: "engine", role: "engine", what: "engine torque table",
                          corner: "RPM \\ Throttle %", event: "engine" });
}

//: Put *table* into the step (the sample at the start).
export const showEngine = table => step.show(table);
