use serde_json::Value;

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
    let cases: Vec<Value> = include_str!("fixtures/numerical-baseline.jsonl")
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(cases.len(), 375);
    for (i, case) in cases.iter().enumerate() {
        let result = openetv::batch::execute(&case["input"]).unwrap();
        assert!(equal_numbers(&case["expected"], &result), "case {i}");
    }
}
