# Calculation

Choose the output breakpoints, then click **Calculate ETV map** to open the result.

Drag a breakpoint to adjust it, or double-click to type. Enter a **New RPM** or **New pedal** value and click **Add RPM** or **Add pedal**. **×** removes a point; at least one point stays on each axis. Values stay in ascending order.

The generator uses these same breakpoints. **Undo** restores a change.

| Field | What it changes |
|---|---|
| **Output RPM breakpoints** | The result's rows. |
| **Output pedal breakpoints (%)** | The result's columns. |
| **RPM calc method** | The speed separating the two maximum-torque treatments. |
| **Max torque tolerance** | How close to the row's maximum counts as maximum torque, in Nm. |

The threshold and tolerance affect saturated cells. For their precise meaning, choose **Calculation reference** in this manual.

**How it works** lets you move RPM and pedal position through the request, throttle target and modelled torque. It uses the current result when it is up to date.
