# ETV map

Each cell is the throttle target at one RPM and grip position. Colours use a fixed 0–100% scale.

## Corrections

| Do this | Get that |
|---|---|
| **Zero-gas fix** | Set closed grip to zero throttle, then join the value at **Ramp to**. |
| **Flat-spot fix** | Replace a plateau in the throttle curve with a ramp. |
| **Remove dips** | Prevent throttle from decreasing as grip increases. |
| **Reset to calculated** | Discard result edits and return to the calculated table. |

These edits change the target. **Undo** restores the previous result.

## Read the result

| You see | It is |
|---|---|
| ▲ | The requested torque reaches or exceeds the saturation threshold. |
| ▽ | The request is below closed-throttle torque. |
| ! | Torque falls below the request again at a larger opening. |
| **Inputs changed** | Recalculate before editing or exporting. |

Symbols describe the original calculation, even after corrections. **Torque comparison** shows modelled torque in cyan and requested torque in amber; it is not a measurement.

Choose **3D map** above the map to rotate the full surface. **Table** returns to cell editing. See **3D maps** for the controls.
