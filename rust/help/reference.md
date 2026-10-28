# Calculation reference

## From torque to throttle

For each output RPM, OpenETV interpolates the engine and request tables. It finds the throttle opening that produces the requested torque by reading the engine curve backwards.

| Condition | Treatment |
|---|---|
| An engine row contains only zeroes | Omit it from engine interpolation. |
| Output RPM lies outside the usable engine rows | Hold the nearest usable row and report the RPM. |
| Request is below the first throttle point's torque | Use the first throttle breakpoint and mark ▽. |
| Request reaches maximum torque minus tolerance | Mark ▲ and use a qualifying throttle breakpoint. |
| RPM is at or below **RPM calc method** | Use the first qualifying breakpoint for saturation. |
| RPM is above **RPM calc method** | Use the last qualifying breakpoint for saturation. |

## Model limits

Recovered requests and torque comparisons use the loaded engine table. They do not measure delivered torque or validate throttle hardware. Engine braking, cylinder cut, ETV gain and multi-gear processing are not included.

CSV, XLSX and DSS inputs are limited to 16 MB and 20,000 table cells.
