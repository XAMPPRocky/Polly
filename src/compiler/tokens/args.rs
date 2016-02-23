use serde_json::Value;
use super::Component;
#[derive(Debug, Clone, PartialEq)]
pub enum ArgKey {
    Json(String),
    Comp(String),
}

impl ArgKey {
    pub fn value(&self) -> String {
        use self::ArgKey::*;
        match *self {
            Json(ref string) | Comp(ref string) => string.clone(),
        }
    }
}

/// Enum representing the values of the arguments passed into a Function, or Component.
#[derive(Debug, Clone, PartialEq)]
pub enum ArgValue {
    /// JSON passed into the Function, or Component.
    Json(Option<Value>),
    /// Component passed into the Function, or Component.
    Comp(Option<Component>),
}
