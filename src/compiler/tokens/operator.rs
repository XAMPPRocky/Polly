use std::fmt;
use std::fmt::{Display, Formatter};

use super::consts::*;

/// The types of operators
#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    /// TODO
    Ampersand,
    /// TODO
    At,
    /// TODO
    BackSlash,
    /// TODO
    CloseBrace,
    /// TODO
    CloseParam,
    /// TODO
    Comma,
    /// TODO
    Dollar,
    /// TODO
    Dot,
    /// TODO
    Equals,
    /// TODO
    ForwardSlash,
    /// TODO
    OpenBrace,
    /// TODO
    OpenParam,
    /// TODO
    Pound,
    /// TODO
    Quote,
    /// TODO
    Star,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::Operator::*;
        let ch = match *self {
            Ampersand => AMPERSAND,
            At => AT,
            BackSlash => BACKSLASH,
            CloseBrace => CLOSEBRACE,
            CloseParam => CLOSEPARAM,
            Comma => COMMA,
            Dollar => DOLLAR,
            Dot => DOT,
            Equals => EQUALS,
            ForwardSlash => FORWARDSLASH,
            OpenBrace => OPENBRACE,
            OpenParam => OPENPARAM,
            Pound => POUND,
            Quote => DOUBLEQUOTE,
            Star => STAR,
        };
        write!(f, "{}", ch)
    }
}
