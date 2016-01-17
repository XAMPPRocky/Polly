use serde_json::Value;
use template::PolyFn;

#[derive(Debug, Clone, PartialEq)]
pub enum Args {
    Generic(Value),
    Component(String),
    Fn(Box<PolyFn>),
}
