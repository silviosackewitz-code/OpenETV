use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
};

fn cases() -> Vec<Value> {
    include_str!("fixtures/numerical-baseline.jsonl")
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn equal_numbers(expected: &Value, actual: &Value) -> bool {
    match (expected, actual) {
        (Value::Number(a), Value::Number(b)) => a.as_f64() == b.as_f64(),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal_numbers(a, b))
        }
        (Value::Object(a), Value::Object(b)) => a
            .iter()
            .all(|(k, a)| b.get(k).is_some_and(|b| equal_numbers(a, b))),
        _ => expected == actual,
    }
}

#[test]
fn frozen_numerical_baseline() {
    let cases = cases();
    assert_eq!(cases.len(), 375);
    for (i, case) in cases.iter().enumerate() {
        let result = openetv::batch::execute(&case["input"]).unwrap();
        assert!(equal_numbers(&case["expected"], &result), "case {i}");
    }
}

#[test]
fn binary_batch_interface_matches_baseline() {
    let cases = cases();
    let input: Vec<_> = cases.iter().map(|c| &c["input"]).collect();
    let binary =
        std::env::var_os("OPENETV_BINARY").unwrap_or_else(|| env!("CARGO_BIN_EXE_openetv").into());
    let mut process = Command::new(binary)
        .arg("--batch")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    process
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&input).unwrap())
        .unwrap();
    let output = process.wait_with_output().unwrap();
    assert!(output.status.success());
    let results: Vec<Value> = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(results.len(), cases.len());
    for (i, (case, result)) in cases.iter().zip(results).enumerate() {
        assert!(equal_numbers(&case["expected"], &result), "binary case {i}");
    }
}
