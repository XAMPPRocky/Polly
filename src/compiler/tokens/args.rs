use super::ast::Token;
use self::Args::*;
use serde_json::Value;
use template::PolyFn;

#[derive(Debug, Clone, PartialEq)]
pub enum Args {
    Text(String),
    Component(String),
}
