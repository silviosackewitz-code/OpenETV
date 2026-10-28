# Torque request

The request says how much torque the rider asks for at each RPM and grip position.

| To… | Choose… |
|---|---|
| Use an existing request in Nm | **Import request…**. |
| Create a request from the engine table | **Generate a torque request**. |
| Inspect what an existing throttle map requests | **Recover request from ETV…**. The result is an estimate from the engine table. |

## Generate a request

1. Open **Generate a torque request** and choose **Shape**.
2. Set the share of maximum torque and the breakpoints. Open **Shape preview** to inspect the grip response.
3. Click **Generate request** to replace the request table. **Undo** restores the previous table.

| Choose | Get that |
|---|---|
| **Scale to maximum at each RPM (torque follows engine speed)** off | The same grip position asks for the same Nm, capped by the available torque. |
| The same option on | Torque follows the engine's maximum at each RPM. |

The curve shows the request in cyan and available engine torque in amber.
