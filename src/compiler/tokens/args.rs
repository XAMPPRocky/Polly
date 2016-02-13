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

#[derive(Debug, Clone, PartialEq)]
pub enum ArgValue {
    Json(Option<Value>),
    Comp(Option<Component>),
}
