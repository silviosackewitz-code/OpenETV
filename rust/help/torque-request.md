# Torque request

The request says how much torque the rider asks for at each RPM and grip position.

| To… | Choose… |
|---|---|
| Use an existing request in Nm | **Import request…**. |
| Create a request from the engine table | **Generate a torque request**. |
| Inspect what an existing throttle map requests | **Recover request from ETV…**. The result is an estimate from the engine table. |

## Generate a request

1. Open **Generate a torque request** and choose **Shape**.
2. Set the share of maximum torque. Drag individual breakpoints or double-click to type. Use **New RPM** / **New pedal** and **Add RPM** / **Add pedal** to insert points; **×** removes them.
3. Open **Shape preview** to inspect the grip response.
4. Click **Generate request** to replace the request table. **Undo** restores the previous table.

| Choose | Get that |
|---|---|
| **Scale to maximum at each RPM (torque follows engine speed)** off | The same grip position asks for the same Nm, capped by the available torque. |
| The same option on | Torque follows the engine's maximum at each RPM. |

The curve shows the request in cyan and available engine torque in amber.

Choose **3D map** above the map to rotate the full surface. **Table** returns to cell editing. See **3D maps** for the controls.
