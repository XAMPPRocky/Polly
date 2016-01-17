use std::fmt;
use std::fmt::{Display, Formatter};
use operator::Operator;
use operator::Operator::*;

/// Parent enum defining the two types of Terminal symbols within the language.
/// Words, and operator symbols.
#[derive(Debug, PartialEq, Clone)]
pub enum Lexeme {
    /// TODO
    Operator(usize, Operator),
    /// TODO
    Word(usize, String),
    Empty,
}

impl Lexeme {
    pub fn length(&self) -> usize {
        use self::Lexeme::*;
        match *self {
            Operator(_, _) => 1,
            Word(_, ref word) => word.len(),
            Empty => 0,
        }
    }

    pub fn index(&self) -> usize {
        use self::Lexeme::*;
        match *self {
            Operator(index, _) => index,
            Word(index, _) => index,
            Empty => unreachable!(),
        }
    }
}

impl Display for Lexeme {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::Lexeme::*;

        let display = match *self {
            Operator(_, ref operator) => format!("OPERATOR: {}", operator.to_string()),
            Word(_, ref word) => format!("WORD: {}", word.clone()),
            Empty => "empty?? Some problem with your comments.".to_owned(),
        };

        write!(f, "{}", display)
    }
}
