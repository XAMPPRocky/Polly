use super::ast::Token;
use self::Args::*;
use serde_json::Value;
use template::PolyFn;

#[derive(Debug, Clone)]
pub enum Args {
    Generic(Value),
    Component(Vec<Token>),
}


impl PartialEq for Args {
    fn eq(&self, other: &Args) -> bool {
        match self {
            &Generic(ref lhs) => {
                if let &Generic(ref rhs) = other {
                    lhs.eq(rhs)
                } else {
                    false
                }
            }
            &Component(ref lhs) => {
                if let &Component(ref rhs) = other {
                    lhs.eq(rhs)
                } else {
                    false
                }
            }
        }
    }
}
