use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub rpm: Vec<f64>,
    pub axis: Vec<f64>,
    pub values: Vec<Vec<f64>>,
}
impl Table {
    pub fn new(rpm: Vec<f64>, axis: Vec<f64>, values: Vec<Vec<f64>>) -> Result<Self, String> {
        if rpm.is_empty()
            || axis.is_empty()
            || values.len() != rpm.len()
            || values.iter().any(|r| r.len() != axis.len())
        {
            return Err("Table dimensions are empty or inconsistent.".into());
        }
        if rpm
            .iter()
            .chain(&axis)
            .chain(values.iter().flatten())
            .any(|x| !x.is_finite())
        {
            return Err("Table cells and breakpoints must be finite numbers.".into());
        }
        let mut rows: Vec<usize> = (0..rpm.len()).collect();
        let mut cols: Vec<usize> = (0..axis.len()).collect();
        rows.sort_by(|&a, &b| rpm[a].total_cmp(&rpm[b]));
        cols.sort_by(|&a, &b| axis[a].total_cmp(&axis[b]));
        let table = Self {
            rpm: rows.iter().map(|&i| rpm[i]).collect(),
            axis: cols.iter().map(|&j| axis[j]).collect(),
            values: rows
                .iter()
                .map(|&i| cols.iter().map(|&j| values[i][j]).collect())
                .collect(),
        };
        table.validate()?;
        Ok(table)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.rpm.is_empty()
            || self.axis.is_empty()
            || self.values.len() != self.rpm.len()
            || self.values.iter().any(|r| r.len() != self.axis.len())
        {
            return Err("Table dimensions are empty or inconsistent.".into());
        }
        if self
            .rpm
            .iter()
            .chain(&self.axis)
            .chain(self.values.iter().flatten())
            .any(|x| !x.is_finite())
        {
            return Err("Table cells and breakpoints must be finite numbers.".into());
        }
        if [&self.rpm, &self.axis]
            .iter()
            .any(|a| a.windows(2).any(|p| p[0] >= p[1]))
        {
            return Err("Breakpoints must be unique and ascending.".into());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Saturated,
    BelowMin,
    NonMonotonic,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EtvResult {
    pub table: Table,
    pub status: Vec<Vec<Status>>,
    pub outside: Vec<f64>,
}
fn rounded(x: f64, digits: i32) -> f64 {
    let scale = 10f64.powi(digits);
    (x * scale).round_ties_even() / scale
}
fn bracket(a: &[f64], x: f64) -> (usize, usize, f64) {
    if a.len() == 1 {
        return (0, 0, 0.0);
    }
    let x = x.clamp(a[0], a[a.len() - 1]);
    let hi = a.partition_point(|v| *v <= x).clamp(1, a.len() - 1);
    (hi - 1, hi, (x - a[hi - 1]) / (a[hi] - a[hi - 1]))
}
pub fn lookup(table: &Table, rpm: f64, x: f64) -> f64 {
    let (i, k, fr) = bracket(&table.rpm, rpm);
    let (j, l, fc) = bracket(&table.axis, x);
    let v = &table.values;
    v[i][j] * (1.0 - fr) * (1.0 - fc)
        + v[i][l] * (1.0 - fr) * fc
        + v[k][j] * fr * (1.0 - fc)
        + v[k][l] * fr * fc
}
pub fn engine_row(engine: &Table, rpm: f64) -> (Vec<f64>, bool) {
    let real: Vec<usize> = engine
        .values
        .iter()
        .enumerate()
        .filter(|(_, r)| r.iter().any(|x| *x != 0.0))
        .map(|(i, _)| i)
        .collect();
    if real.is_empty() {
        return (engine.values[0].clone(), false);
    }
    let first = real[0];
    let last = *real.last().unwrap();
    if rpm <= engine.rpm[first] || rpm >= engine.rpm[last] {
        let i = if rpm <= engine.rpm[first] {
            first
        } else {
            last
        };
        return (engine.values[i].clone(), rpm != engine.rpm[i]);
    }
    let hi = real.partition_point(|&i| engine.rpm[i] <= rpm);
    let (lo, hi) = (real[hi - 1], real[hi]);
    let f = (rpm - engine.rpm[lo]) / (engine.rpm[hi] - engine.rpm[lo]);
    (
        engine.values[lo]
            .iter()
            .zip(&engine.values[hi])
            .map(|(a, b)| a * (1.0 - f) + b * f)
            .collect(),
        false,
    )
}
fn breakpoints(a: &[f64]) -> Result<(), String> {
    if a.is_empty() || a.iter().any(|x| !x.is_finite()) || a.windows(2).any(|p| p[0] >= p[1]) {
        Err("Output breakpoints must be finite, unique and ascending.".into())
    } else {
        Ok(())
    }
}
pub fn calculate(
    engine: &Table,
    request: &Table,
    rpm: &[f64],
    grip: &[f64],
    threshold: f64,
    tolerance: f64,
) -> Result<EtvResult, String> {
    engine.validate()?;
    request.validate()?;
    breakpoints(rpm)?;
    breakpoints(grip)?;
    if !threshold.is_finite() || !tolerance.is_finite() || tolerance < 0.0 {
        return Err("Threshold must be finite and tolerance nonnegative.".into());
    }
    let mut values = Vec::new();
    let mut status = Vec::new();
    let mut outside = Vec::new();
    for &r in rpm {
        let (torque, out) = engine_row(engine, r);
        if out {
            outside.push(r)
        }
        let max = torque.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let mut row = Vec::new();
        let mut states = Vec::new();
        for &g in grip {
            let target = lookup(request, r, g);
            let (x, s) = if target >= max - tolerance {
                let mut near = torque
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| **t >= max - tolerance)
                    .map(|(i, _)| i);
                let i = if r > threshold {
                    near.next_back().unwrap()
                } else {
                    near.next().unwrap()
                };
                (engine.axis[i], Status::Saturated)
            } else if target <= torque[0] {
                (engine.axis[0], Status::BelowMin)
            } else {
                let j = torque.iter().position(|t| *t >= target).unwrap();
                (
                    engine.axis[j - 1]
                        + (target - torque[j - 1]) / (torque[j] - torque[j - 1])
                            * (engine.axis[j] - engine.axis[j - 1]),
                    if torque[j..].iter().any(|t| *t < target) {
                        Status::NonMonotonic
                    } else {
                        Status::Ok
                    },
                )
            };
            row.push(rounded(x, 1));
            states.push(s)
        }
        values.push(row);
        status.push(states)
    }
    Ok(EtvResult {
        table: Table::new(rpm.to_vec(), grip.to_vec(), values)?,
        status,
        outside,
    })
}
pub fn zero_gas_fix(etv: &Table, ramp_to: f64) -> Result<Table, String> {
    etv.validate()?;
    if !ramp_to.is_finite() || ramp_to < 0.0 {
        return Err("Ramp endpoint must be finite and nonnegative.".into());
    }
    let mut out = etv.clone();
    for row in &mut out.values {
        for (j, g) in etv.axis.iter().enumerate() {
            if *g == 0.0 {
                row[j] = 0.0
            }
        }
    }
    if ramp_to > 0.0
        && let Some(end) = etv
            .axis
            .iter()
            .enumerate()
            .filter(|(_, x)| **x > 0.0)
            .min_by(|(_, a), (_, b)| (*a - ramp_to).abs().total_cmp(&(*b - ramp_to).abs()))
            .map(|(i, _)| i)
    {
        for row in &mut out.values {
            for j in 0..end {
                if etv.axis[j] > 0.0 {
                    row[j] = rounded(row[end] * (etv.axis[j] / etv.axis[end]), 1)
                }
            }
        }
    }
    Ok(out)
}
pub fn flat_spot_fix(etv: &Table) -> Table {
    let mut out = etv.clone();
    for row in &mut out.values {
        let mut end = row.len() - 1;
        while end > 0 {
            let mut start = end;
            while start > 0 && row[start - 1] == row[end] {
                start -= 1
            }
            if start < end && start > 0 {
                let lo = start - 1;
                for j in start..end {
                    row[j] = rounded(
                        row[lo]
                            + (etv.axis[j] - etv.axis[lo]) / (etv.axis[end] - etv.axis[lo])
                                * (row[end] - row[lo]),
                        1,
                    )
                }
            }
            if start == 0 {
                break;
            }
            end = start - 1
        }
    }
    out
}
pub fn monotonic_fix(etv: &Table) -> Table {
    let mut out = etv.clone();
    for row in &mut out.values {
        for j in 1..row.len() {
            row[j] = row[j].max(row[j - 1])
        }
    }
    out
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Shape {
    Power { n: f64 },
    SCurve { center: f64, steepness: f64 },
}
impl Shape {
    pub fn validate(&self) -> Result<(), String> {
        match *self {
            Self::Power { n } if n.is_finite() && n >= 0.05 => Ok(()),
            Self::SCurve { center, steepness }
                if center.is_finite()
                    && (1.0..=99.0).contains(&center)
                    && steepness.is_finite()
                    && steepness >= 0.5 =>
            {
                Ok(())
            }
            _ => Err("Invalid request shape parameters.".into()),
        }
    }
    pub fn value(&self, x: f64) -> f64 {
        match *self {
            Self::Power { n } => x.powf(n),
            Self::SCurve { center, steepness } => {
                let c = center / 100.0;
                let lo = 1.0 / (1.0 + (steepness * c).exp());
                let hi = 1.0 / (1.0 + (-steepness * (1.0 - c)).exp());
                (1.0 / (1.0 + (-steepness * (x - c)).exp()) - lo) / (hi - lo)
            }
        }
    }
}
pub fn generate_request(
    engine: &Table,
    rpm: &[f64],
    grip: &[f64],
    shape: Shape,
    max_percent: f64,
    per_rpm: bool,
) -> Result<Table, String> {
    engine.validate()?;
    breakpoints(rpm)?;
    breakpoints(grip)?;
    shape.validate()?;
    if !max_percent.is_finite() || max_percent < 0.0 {
        return Err("Maximum percent must be finite and nonnegative.".into());
    }
    let mut real: Vec<usize> = engine
        .values
        .iter()
        .enumerate()
        .filter(|(_, r)| r.iter().any(|x| *x != 0.0))
        .map(|(i, _)| i)
        .collect();
    if real.is_empty() {
        real = (0..engine.rpm.len()).collect()
    }
    let rr: Vec<f64> = real.iter().map(|&i| engine.rpm[i]).collect();
    let maxima: Vec<f64> = real
        .iter()
        .map(|&i| {
            engine.values[i]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max)
        })
        .collect();
    let full = engine
        .values
        .iter()
        .flatten()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max)
        * (max_percent / 100.0);
    let values = rpm
        .iter()
        .map(|r| {
            let (a, b, f) = bracket(&rr, *r);
            let max = maxima[a] + f * (maxima[b] - maxima[a]);
            grip.iter()
                .map(|g| {
                    let share = shape.value(*g / 100.0);
                    rounded(
                        if per_rpm {
                            max * (max_percent / 100.0) * share
                        } else {
                            (full * share).min(max)
                        },
                        2,
                    )
                })
                .collect()
        })
        .collect();
    Table::new(rpm.to_vec(), grip.to_vec(), values)
}
pub fn request_from_etv(engine: &Table, etv: &Table) -> Result<Table, String> {
    engine.validate()?;
    etv.validate()?;
    let values = etv
        .rpm
        .iter()
        .zip(&etv.values)
        .map(|(&rpm, row)| {
            let (torque, _) = engine_row(engine, rpm);
            row.iter()
                .map(|&throttle| {
                    let value = if throttle <= engine.axis[0] {
                        torque[0]
                    } else if throttle >= engine.axis[engine.axis.len() - 1] {
                        torque[torque.len() - 1]
                    } else {
                        let hi = engine.axis.partition_point(|x| *x <= throttle);
                        let lo = hi - 1;
                        // Match one-dimensional interpolation before decimal rounding.
                        let slope = (torque[hi] - torque[lo]) / (engine.axis[hi] - engine.axis[lo]);
                        torque[lo] + slope * (throttle - engine.axis[lo])
                    };
                    rounded(value, 2)
                })
                .collect()
        })
        .collect();
    Table::new(etv.rpm.clone(), etv.axis.clone(), values)
}
