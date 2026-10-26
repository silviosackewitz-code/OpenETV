# Native desktop migration

The desktop window now uses Rust and egui instead of a local web server and embedded browser. The native interface keeps the five-step engine-table, request, calculation, result and export workflow. It uses the shared Race Analysis typography and palette conventions.

The calculation and file handling remain local. CSV, XLSX and Mectronik DSS inputs and exports retain explicit table units. The migration preserves the established calculation behaviour rather than introducing new calibration defaults.

## Workflow additions

- Recover a modelled torque request from an existing ETV throttle map and the loaded engine table.
- Save and open complete JSON .openetv sessions. Recalculate a loaded result before export.
- Undo and redo the latest 40 actions, including table edits and result adjustments.
- Mark results stale when their inputs or calculation settings change; require recalculation before export.
- Inspect table heatmaps and line charts, with labelled table and comparison series.
- Retain system, dark and light themes and an accent preference in the existing preferences file.

The in-app manual is embedded in the binary from MANUAL.md. Manual screenshots have not been produced for the native interface.

## Model and scope

A recovered request and any delivered-torque comparison are estimates from the loaded engine table. They do not measure delivered torque and do not validate an ECU's throttle actuator or other torque-control interventions. A calculated map still requires dyno validation before riding.

This migration does not add hardware control, automatic ECU flashing, guessed actuator limits, automatic map smoothing or a new engine-braking model. No manufacturer numerical settings are imported as defaults.

## Public research

[MaxxECU throttle targets](https://www.maxxecu.se/webhelp/settings-ethrottle-throttle_targets.html) describes pedal-to-throttle mapping. This supports distinguishing throttle-percent tables from torque requests in Nm.

[MaxxECU throttle control](https://www.maxxecu.se/webhelp/settings-ethrottle-throttle_control.html) distinguishes target and actual throttle channels and documents hardware calibration and controller checks. An offline target table covers only part of that system.

[MoTeC package release notes](https://moteconline.motec.com.au/Package/ViewVariant?PackageVariantId=4982) distinguish a pedal torque aim referencing current maximum torque from one referencing a torque limit. These are different behaviours; generator modes therefore need explicit labels. These notes describe a specific package, not every MoTeC firmware.

[Haltech torque management](https://support.haltech.com/portal/en/kb/articles/torque-management-elite) describes a drag-racing RPM control strategy. Its similarly named function is not evidence for changing OpenETV's table inversion or calibration values.
