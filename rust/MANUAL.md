# OpenETV manual

OpenETV builds a throttle target map from an engine torque table and a rider torque request. The engine table uses RPM and throttle position (%) to describe torque (Nm). The request uses RPM and grip position (%) to describe requested torque (Nm). The result uses RPM and grip position (%) to describe throttle position (%).

## 1. Engine torque

Load the engine's measured torque table from CSV, Excel (.xlsx), or a Mectronik .dss file. For a file containing several tables, choose the intended table. Check the table name, axes and units before continuing: a throttle map in percent is not a torque table in Nm.

Tables can be edited directly. Select a cell and paste a rectangular block from a spreadsheet. Table tools can fill, add an offset, multiply or interpolate a selected row or column. Interpolation follows the numerical breakpoints and keeps both endpoints. Result axes are protected; change them in Calculation. New columns interpolate at the largest gap. Invalid edits leave the table unchanged. Charts show the selected table slice; inspect the shape and the breakpoints, not just individual values. Imported data describes the engine used to measure it. It is not a substitute for measurements of a different engine or configuration.

## 2. Torque request

Load a request in Nm, or generate one from the engine table. The power-shaped generator and the custom S-curve generator change how torque grows with grip travel. Absolute torque uses one engine-wide reference; the per-RPM mode scales the request against the torque available at each engine speed. Inspect the preview before replacing the request.

To inspect an existing throttle map, import its ETV table and recover the corresponding torque request using the loaded engine table. This forward calculation estimates the torque represented by that map; it does not measure the engine.

## 3. Calculation

Review the output RPM and grip breakpoints. RPM and grip axes must be ordered and have distinct values. Calculation uses the engine table to find the throttle opening for each requested torque. Requests outside the engine's available range can saturate; RPM outside the measured range relies on boundary data and needs particular attention.

The RPM calculation threshold selects the engine-speed region where the chosen maximum-torque treatment applies. Maximum-torque tolerance controls how close an opening must be to the row's maximum torque to qualify for that treatment. These settings affect the resulting map. Review their effect in the result rather than treating a successful calculation as validation.

## 4. ETV map

Inspect the calculated throttle table and its curves. Chart labels identify the quantities: cyan is the displayed table, amber is the comparison or overlay. Follow axis labels and units when comparing them.

The zero-gas adjustment sets the closed-grip boundary and joins it to the selected end of the low-grip ramp. Flat-spot adjustment replaces throttle plateaus with a ramp. Removing dips makes throttle nondecreasing as grip increases. These are distinct edits and can change the torque delivered by the model. Reset to the raw calculation to discard result edits and adjustments.

Edit result cells directly when necessary. Undo and redo retain the latest 40 actions. A stale-result warning means inputs or calculation settings changed after the result was made. Recalculate before exporting.

## 5. Export

Choose CSV, Excel (.xlsx), or Mectronik .dss and specify the destination path. Export writes the result table: RPM, grip position (%) and throttle position (%). It does not write a torque request in Nm. Confirm the destination, ECU table and axes before using the exported file.

A calculated map must be validated on a dyno before riding. Check its behaviour across grip positions and engine speeds. Offline table calculation does not validate throttle hardware, sensor calibration, ECU control strategies or the actual engine response.

## Sessions and shortcuts

Save a session as a JSON .openetv file to retain inputs, settings and result together. Open a session to continue that work. Save refuses to overwrite a project changed by another application; use Save as with another filename. When opening a project, OpenETV recomputes and verifies the saved baseline. Matching projects retain deliberate result corrections; changed inputs or a mismatched baseline require recalculation before export.

Command-S saves a session. Command-Z undoes an action; Command-Shift-Z redoes it. On platforms using Control for standard shortcuts, use Control instead of Command.

## Appearance

Choose System, Dark, Light, Industrial or Terminal, IBM Plex or JetBrains Mono, and an interface size. Dark and Light use the selected accent; Industrial and Terminal keep their signal colour. Preferences are saved in ~/.openetv/prefs.json, preserving the existing preferences format. The native window follows the shared Race Analysis font and palette conventions. Appearance settings do not change table values.
