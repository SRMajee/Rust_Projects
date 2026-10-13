use json_dsl::{json, JsonValue};
use std::collections::HashMap;

#[test]
fn test_json_primitives() {
    assert_eq!(json!(null), JsonValue::Null);
    assert_eq!(json!(true), JsonValue::Bool(true));
    assert_eq!(json!(false), JsonValue::Bool(false));
    assert_eq!(json!(42), JsonValue::Number(42.0));
    assert_eq!(json!(-17.5), JsonValue::Number(-17.5));
    assert_eq!(json!("hello"), JsonValue::String("hello".to_string()));
    assert_eq!(
        json!(String::from("world")),
        JsonValue::String("world".to_string())
    );
}

#[test]
fn test_json_arrays() {
    // Empty array
    assert_eq!(json!([]), JsonValue::Array(vec![]));

    // Single and multi element
    assert_eq!(json!([1]), JsonValue::Array(vec![JsonValue::Number(1.0)]));
    assert_eq!(
        json!([1, 2, 3]),
        JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::Number(2.0),
            JsonValue::Number(3.0),
        ])
    );

    // Trailing comma
    assert_eq!(
        json!([1, 2,]),
        JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)])
    );

    // Heterogeneous array
    assert_eq!(
        json!([1, "text", true, null]),
        JsonValue::Array(vec![
            JsonValue::Number(1.0),
            JsonValue::String("text".to_string()),
            JsonValue::Bool(true),
            JsonValue::Null,
        ])
    );

    // Nested arrays
    assert_eq!(
        json!([[1, 2], [3, 4]]),
        JsonValue::Array(vec![
            JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]),
            JsonValue::Array(vec![JsonValue::Number(3.0), JsonValue::Number(4.0)]),
        ])
    );
}

#[test]
fn test_json_objects() {
    // Empty object
    assert_eq!(json!({}), JsonValue::Object(HashMap::new()));

    // Quoted keys
    let mut expected = HashMap::new();
    expected.insert("name".to_string(), JsonValue::String("Alice".to_string()));
    expected.insert("age".to_string(), JsonValue::Number(30.0));
    assert_eq!(json!({ "name": "Alice", "age": 30 }), JsonValue::Object(expected));

    // Unquoted ident keys
    let mut expected_ident = HashMap::new();
    expected_ident.insert("active".to_string(), JsonValue::Bool(true));
    assert_eq!(json!({ active: true }), JsonValue::Object(expected_ident));

    // Trailing comma in object
    let mut expected_trailing = HashMap::new();
    expected_trailing.insert("k".to_string(), JsonValue::Number(1.0));
    assert_eq!(json!({ "k": 1, }), JsonValue::Object(expected_trailing));
}

#[test]
fn test_nested_complex_structures() {
    let doc = json!({
        "status": "success",
        "code": 200,
        "result": {
            "items": [
                { "id": 1, "name": "item_1" },
                { "id": 2, "name": "item_2" }
            ],
            "total": 2,
            "cursor": null
        }
    });

    assert_eq!(doc.get("status").and_then(|v| v.as_str()), Some("success"));
    assert_eq!(doc.get("code").and_then(|v| v.as_f64()), Some(200.0));

    let result = doc.get("result").expect("expected result object");
    assert_eq!(result.get("total").and_then(|v| v.as_f64()), Some(2.0));
    assert!(result.get("cursor").expect("expected cursor").is_null());

    let items = result.get("items").and_then(|v| v.as_array()).expect("expected items array");
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].get("id").and_then(|v| v.as_f64()), Some(1.0));
    assert_eq!(items[1].get("name").and_then(|v| v.as_str()), Some("item_2"));
}

#[test]
fn test_variable_and_expression_injection() {
    let base = 100;
    let multiplier = 2;
    let tag = "production";

    let payload = json!({
        "calc": (base * multiplier),
        "environment": tag,
        "is_prod": (tag == "production"),
    });

    assert_eq!(payload.get("calc").and_then(|v| v.as_f64()), Some(200.0));
    assert_eq!(payload.get("environment").and_then(|v| v.as_str()), Some("production"));
    assert_eq!(payload.get("is_prod").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn test_display_serialization() {
    let doc = json!({
        "a": 1,
        "b": "text",
        "c": true,
        "d": null,
        "e": [10, 20]
    });

    // Object keys are deterministically sorted alphabetically by Display
    let rendered = format!("{}", doc);
    let expected = r#"{"a": 1, "b": "text", "c": true, "d": null, "e": [10, 20]}"#;
    assert_eq!(rendered, expected);
}
