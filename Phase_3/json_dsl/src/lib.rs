use std::collections::HashMap;
use std::fmt;

/// Represents any valid JSON value in a recursive AST structure.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self { JsonValue::Object(m) => m.get(key), _ => None }
    }
    pub fn get_index(&self, i: usize) -> Option<&JsonValue> {
        match self { JsonValue::Array(a) => a.get(i), _ => None }
    }
    pub fn is_null(&self) -> bool { matches!(self, JsonValue::Null) }
    pub fn as_str(&self) -> Option<&str> {
        match self { JsonValue::String(s) => Some(s), _ => None }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self { JsonValue::Number(n) => Some(*n), _ => None }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self { JsonValue::Bool(b) => Some(*b), _ => None }
    }
    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self { JsonValue::Array(a) => Some(a), _ => None }
    }
    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self { JsonValue::Object(m) => Some(m), _ => None }
    }
}

impl From<bool> for JsonValue { fn from(b: bool) -> Self { JsonValue::Bool(b) } }
impl From<&str> for JsonValue { fn from(s: &str) -> Self { JsonValue::String(s.to_string()) } }
impl From<String> for JsonValue { fn from(s: String) -> Self { JsonValue::String(s) } }

macro_rules! impl_from_num {
    ($($t:ty),*) => {
        $(impl From<$t> for JsonValue { fn from(n: $t) -> Self { JsonValue::Number(n as f64) } })*
    };
}
impl_from_num!(f64, f32, i8, i16, i32, i64, u8, u16, u32, u64, usize);

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Bool(b) => write!(f, "{b}"),
            JsonValue::Number(n) => {
                if n.fract() == 0.0 && !n.is_infinite() && !n.is_nan() && *n >= (i64::MIN as f64) && *n <= (i64::MAX as f64) {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            JsonValue::String(s) => {
                write!(f, "\"")?;
                for ch in s.chars() {
                    match ch {
                        '"' => write!(f, "\\\"")?,
                        '\\' => write!(f, "\\\\")?,
                        '\n' => write!(f, "\\n")?,
                        '\t' => write!(f, "\\t")?,
                        '\r' => write!(f, "\\r")?,
                        c => write!(f, "{c}")?,
                    }
                }
                write!(f, "\"")
            }
            JsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            JsonValue::Object(map) => {
                write!(f, "{{")?;
                let mut entries: Vec<_> = map.iter().collect();
                entries.sort_by_key(|&(k, _)| k);
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "\"{k}\": {v}")?;
                }
                write!(f, "}}")
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! json_key {
    ($key:literal) => { $key.to_string() };
    ($key:ident) => { stringify!($key).to_string() };
    ($key:expr) => { $key.to_string() };
}

/// Declarative JSON DSL macro that recursively parses nested key-value and array syntax
/// directly into a `JsonValue` AST at compile time.
///
/// ```
/// use json_dsl::json;
///
/// let val = json!({
///     "name": "Alice",
///     "tags": ["rust", "macro"],
///     "verified": true,
///     "score": 99.5,
///     "meta": null
/// });
/// assert_eq!(val.get("name").and_then(|v| v.as_str()), Some("Alice"));
/// ```

#[macro_export]
macro_rules! json {
    (null) => { $crate::JsonValue::Null };
    ([]) => { $crate::JsonValue::Array(::std::vec::Vec::new()) };
    ([ $( $elem:tt ),* $(,)? ]) => {
        $crate::JsonValue::Array(::std::vec![ $( $crate::json!($elem) ),* ])
    };
    ({}) => { $crate::JsonValue::Object(::std::collections::HashMap::new()) };
    ({ $( $key:tt : $val:tt ),* $(,)? }) => {{
        let mut map = ::std::collections::HashMap::new();
        $( map.insert($crate::json_key!($key), $crate::json!($val)); )*
        $crate::JsonValue::Object(map)
    }};
    ($other:expr) => { $crate::JsonValue::from($other) };
}