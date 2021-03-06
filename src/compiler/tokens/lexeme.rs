use std::fmt;
use std::fmt::{Display, Formatter};

use self::Lexeme::*;
use super::operator::Operator;

/// Parent enum defining the two types of Terminal symbols within the language.
/// Words, and operator symbols.
#[derive(Debug, PartialEq, Clone)]
pub enum Lexeme {
    Symbol(usize, Operator),
    Word(usize, String),
}

impl Lexeme {
    pub fn length(&self) -> usize {
        match *self {
            Symbol(_, _) => 1,
            Word(_, ref word) => word.len(),
        }
    }

    pub fn index(&self) -> usize {
        match *self {
            Symbol(index, _) | Word(index, _) => index,
        }
    }
}

impl Display for Lexeme {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let display = match *self {
            Symbol(_, ref operator) => format!("OPERATOR: {}", operator.to_string()),
            Word(_, ref word) => format!("WORD: {}", word.clone()),
        };

        write!(f, "{}", display)
    }
}
