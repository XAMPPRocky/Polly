use std::fmt;
use std::fmt::{Display, Formatter};
use std::error;

use super::{ComponentCall, Element, FunctionCall, Lexeme};
use self::AstError::*;

/// TODO
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// TODO
    Html(Element),
    /// TODO
    Text(String),
    /// TODO
    Variable(String),
    /// TODO
    CompCall(ComponentCall),
    /// TODO
    Function(FunctionCall),
}

/// Errors defining all the errors that can be encountered while parsing.
#[derive(Debug, PartialEq, Clone)]
pub enum AstError {
    /// End of File
    Eof,
    /// Expected a Component name.
    ExpectedCompCall(Lexeme),
    /// Expected a Variable name.
    ExpectedVariable(Lexeme),
    /// No name attached to component.
    InvalidComponent(Lexeme),
    /// No name attached to element.
    InvalidElement(Lexeme),
    /// No name attached to function.
    InvalidFunctionCall(Lexeme),
    /// Token that isn't (, ), =, ", ', or a word. 
    InvalidTokenInAttributes(Lexeme),
    /// Having a . without anything following it up.
    NoNameAttachedToClass(Lexeme),
    /// Having a # without anything following it up.
    NoNameAttachedToId(Lexeme),
    /// Extra } braces
    UnclosedCloseBraces(usize),
    /// Extra { braces
    UnclosedOpenBraces(usize),
    /// File ended while we tried to parse element.
    UnexpectedEof(Lexeme),
    /// Unknown token
    UnexpectedToken(Lexeme),
}

impl AstError {
    /// The length of a token, for error printing.
    pub fn values(&self) -> (usize, usize) {
        match *self {
            Eof => (0, 0),
            ExpectedCompCall(ref lexeme) |
            ExpectedVariable(ref lexeme) |
            InvalidComponent(ref lexeme) |
            InvalidElement(ref lexeme) |
            InvalidFunctionCall(ref lexeme) |
            InvalidTokenInAttributes(ref lexeme) |
            NoNameAttachedToClass(ref lexeme) |
            NoNameAttachedToId(ref lexeme) |
            UnexpectedEof(ref lexeme) |
            UnexpectedToken(ref lexeme) => (lexeme.index(), lexeme.length()),
            UnclosedCloseBraces(index) | UnclosedOpenBraces(index) => (index, 1),
        }
    }
}

impl error::Error for AstError {
    fn description(&self) -> &str {
        match *self {
            Eof => "The file ended normally.",
            ExpectedCompCall(_) | InvalidComponent(_) => "Component names can only be words.",
            ExpectedVariable(_) => "Variable names can only be words.",
            InvalidElement(_) => "Element names can only be words.",
            InvalidFunctionCall(_) => "Function names can only be words.",
            InvalidTokenInAttributes(_) => {
                "Attributes fields only accept words as single value, or as key-value word pairs, \
                 or a \") which ends the attributes.\""
            }
            NoNameAttachedToClass(_) => "Class names can only be words.",
            NoNameAttachedToId(_) => "Id names can only be words.",
            UnclosedCloseBraces(_) => "You have an extra closing brace.",
            UnclosedOpenBraces(_) => "You have an extra open brace.",
            UnexpectedEof(_) => "File ended before an element is finished being parsed",
            UnexpectedToken(_) => "Unknown token in use.",
        }
    }
}


impl Display for AstError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use std::error::Error;
        let lexeme = match *self {
            Eof => return write!(f, "{}", self.description()),
            ExpectedCompCall(ref lexeme) |
            ExpectedVariable(ref lexeme) |
            InvalidComponent(ref lexeme) |
            InvalidElement(ref lexeme) |
            InvalidFunctionCall(ref lexeme) |
            InvalidTokenInAttributes(ref lexeme) |
            NoNameAttachedToClass(ref lexeme) |
            NoNameAttachedToId(ref lexeme) |
            UnexpectedEof(ref lexeme) |
            UnexpectedToken(ref lexeme) => lexeme,
            UnclosedCloseBraces(_) | UnclosedOpenBraces(_) => {
                return write!(f, "{}", self.description())
            }

        };
        write!(f, "{}, Got: {}", self.description(), lexeme)
    }
}
