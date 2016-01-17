use std::fmt;
use std::fmt::{Display, Formatter};

use consts::*;

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
    Newline,
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
            Ampersand => AMPERSAND.to_string(),
            At => AT.to_string(),
            BackSlash => BACKSLASH.to_string(),
            CloseBrace => CLOSEBRACE.to_string(),
            CloseParam => CLOSEPARAM.to_string(),
            Comma => COMMA.to_string(),
            Dollar => DOLLAR.to_string(),
            Dot => DOT.to_string(),
            Equals => EQUALS.to_string(),
            ForwardSlash => FORWARDSLASH.to_string(),
            Newline => NEWLINE.to_string(),
            OpenBrace => OPENBRACE.to_string(),
            OpenParam => OPENPARAM.to_string(),
            Pound => POUND.to_string(),
            Quote => DOUBLEQUOTE.to_string(),
            Star => STAR.to_string(),
        };
        write!(f, "{}", ch)
    }
}
