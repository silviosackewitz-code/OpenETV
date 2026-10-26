use crate::model::{self, Shape, Table};
use serde_json::{Value, json};
fn table(v: &Value, key: &str) -> Result<Table, String> {
    let t: Table = serde_json::from_value(v[key].clone()).map_err(|e| e.to_string())?;
    Table::new(t.rpm, t.axis, t.values)
}
fn number(v: &Value, key: &str, default: f64) -> Result<f64, String> {
    match v.get(key) {
        None => Ok(default),
        Some(value) => value
            .as_f64()
            .filter(|x| x.is_finite())
            .ok_or_else(|| format!("Invalid {key}")),
    }
}
fn axis(v: &Value, key: &str, default: &[f64]) -> Result<Vec<f64>, String> {
    match v.get(key) {
        None => Ok(default.to_vec()),
        Some(value) => serde_json::from_value(value.clone()).map_err(|e| e.to_string()),
    }
}
pub fn execute(v: &Value) -> Result<Value, String> {
    match v["op"].as_str().unwrap_or("") {
        "parse" => {
            let path = v["path"].as_str().ok_or("Missing path")?;
            let imported = crate::formats::read_file(std::path::Path::new(path))?;
            Ok(
                json!({"tables":imported.into_iter().map(|t|{let allowed=t.require_torque().is_ok();json!({"path":t.path,"unit":t.unit,"rpm_path":t.rpm_path,"rpm_unit":t.rpm_unit,"other_path":t.other_path,"other_unit":t.other_unit,"table":t.table,"torque_allowed":allowed})}).collect::<Vec<_>>()}),
            )
        }
        "calculate" => {
            let engine = table(v, "engine")?;
            let request = table(v, "request")?;
            let result = model::calculate(
                &engine,
                &request,
                &axis(v, "rpm", &request.rpm)?,
                &axis(v, "grip", &request.axis)?,
                number(v, "rpm_threshold", 8000.)?,
                number(v, "tolerance", 0.3)?,
            )?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "generate" => {
            let engine = table(v, "engine")?;
            let shape: Shape =
                serde_json::from_value(v["shape"].clone()).map_err(|e| e.to_string())?;
            Ok(
                json!({"table":model::generate_request(&engine,&axis(v,"rpm",&engine.rpm)?,&axis(v,"grip",&engine.axis)?,shape,number(v,"max_percent",100.)?,v["per_rpm"].as_bool().unwrap_or(false))?}),
            )
        }
        "postprocess" => {
            let t = table(v, "table")?;
            let result = match v["fix"].as_str().unwrap_or("") {
                "zero_gas" => model::zero_gas_fix(&t, number(v, "ramp_to", 20.)?)?,
                "flat_spots" => model::flat_spot_fix(&t),
                "dips" => model::monotonic_fix(&t),
                _ => return Err("Unknown postprocess fix".into()),
            };
            let changed = result
                .values
                .iter()
                .flatten()
                .zip(t.values.iter().flatten())
                .filter(|(a, b)| a != b)
                .count();
            Ok(json!({"table":result,"changed":changed}))
        }
        "reverse" => {
            Ok(json!({"table":model::request_from_etv(&table(v,"engine")?,&table(v,"table")?)?}))
        }
        _ => Err("Unknown batch operation".into()),
    }
}
