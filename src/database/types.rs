use std::fmt::{self, Display};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Represents a value that can be stored in a database
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    DateTime(DateTime<Utc>),
    Array(Vec<Value>),
    Object(serde_json::Map<String, serde_json::Value>),
    Binary(Vec<u8>),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::DateTime(dt) => write!(f, "\"{}\"", dt),
            Value::Array(_) => write!(f, "[...]"),
            Value::Object(_) => write!(f, "{{...}}"),
            Value::Binary(_) => write!(f, "<binary>"),
        }
    }
}

impl From<&serde_json::Value> for Value {
    fn from(json: &serde_json::Value) -> Self {
        match json {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Boolean(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Float(f)
                } else {
                    Value::String(n.to_string())
                }
            },
            serde_json::Value::String(s) => {
                // Try to parse as DateTime
                if let Ok(dt) = s.parse::<DateTime<Utc>>() {
                    Value::DateTime(dt)
                } else {
                    Value::String(s.clone())
                }
            },
            serde_json::Value::Array(a) => {
                Value::Array(a.iter().map(|v| Value::from(v)).collect())
            },
            serde_json::Value::Object(o) => {
                Value::Object(o.clone())
            },
        }
    }
}

impl From<Value> for serde_json::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => serde_json::Value::Null,
            Value::Boolean(b) => serde_json::Value::Bool(b),
            Value::Integer(i) => serde_json::Value::Number(serde_json::Number::from(i)),
            Value::Float(f) => {
                if let Some(n) = serde_json::Number::from_f64(f) {
                    serde_json::Value::Number(n)
                } else {
                    serde_json::Value::Null
                }
            },
            Value::String(s) => serde_json::Value::String(s),
            Value::DateTime(dt) => serde_json::Value::String(dt.to_rfc3339()),
            Value::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(|v| v.into()).collect())
            },
            Value::Object(obj) => serde_json::Value::Object(obj),
            Value::Binary(bytes) => {
                // Convert binary to base64 string
                let base64 = base64::encode(&bytes);
                serde_json::Value::String(base64)
            },
        }
    }
}
