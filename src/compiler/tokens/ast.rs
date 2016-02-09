use std::fmt;
use std::fmt::{Display, Formatter};
use std::error;

use super::{Component, ComponentCall, Element, FunctionCall, Lexeme};
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
    Comp(Component),
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
    /// Token that isn't ", ', or a word. 
    InvalidTokenAfterEqualsAttributes(Lexeme),
    /// Token that isn't ), =, or a word. 
    InvalidTokenAfterWordInAttributes(Lexeme),
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
    /// Attempted to define a component, where it wasn't valid.
    UnexpectedCompDefine(Lexeme),
    /// File ended while we tried to parse element.
    UnexpectedEof(Lexeme),
    /// Unknown token
    UnexpectedToken(Lexeme),
}

impl AstError {
    pub fn values(&self) -> (usize, usize) {
        match *self {
            Eof => (0, 0),
            ExpectedCompCall(ref lexeme) => (lexeme.index(), lexeme.length()),
            ExpectedVariable(ref lexeme) => (lexeme.index(), lexeme.length()),
            InvalidComponent(ref lexeme) => (lexeme.index(), lexeme.length()),
            InvalidElement(ref lexeme) => (lexeme.index(), lexeme.length()),
            InvalidFunctionCall(ref lexeme) => (lexeme.index(), lexeme.length()),
            InvalidTokenAfterEqualsAttributes(ref lexeme) => (lexeme.index(), lexeme.length()),
            InvalidTokenAfterWordInAttributes(ref lexeme) => (lexeme.index(), lexeme.length()),
            InvalidTokenInAttributes(ref lexeme) => (lexeme.index(), lexeme.length()),
            NoNameAttachedToClass(ref lexeme) => (lexeme.index(), lexeme.length()),
            NoNameAttachedToId(ref lexeme) => (lexeme.index(), lexeme.length()),
            UnclosedCloseBraces(index) => (index, 1),
            UnclosedOpenBraces(index) => (index, 1),
            UnexpectedCompDefine(ref lexeme) => (lexeme.index(), lexeme.length()),
            UnexpectedEof(ref lexeme) => (lexeme.index(), lexeme.length()),
            UnexpectedToken(ref lexeme) => (lexeme.index(), lexeme.length()),
        }
    }
}

impl error::Error for AstError {
    fn description(&self) -> &str {
        match *self {
            Eof => "The file ended normally.",
            ExpectedCompCall(_) => "Component names can only be words.",
            ExpectedVariable(_) => "Variable names can only be words.",
            InvalidComponent(_) => "Element names can only be words.",
            InvalidElement(_) => "Element names can only be words.",
            InvalidFunctionCall(_) => "Function names can only be words.",
            InvalidTokenAfterEqualsAttributes(_) => "Expected quotes after equals.",
            InvalidTokenAfterWordInAttributes(_) => "Unexpected token after a key word.",
            InvalidTokenInAttributes(_) => {
                "Attributes fields only accept words as single value, or as key-value word pairs, \
                 or a \") which ends the attributes.\""
            }
            NoNameAttachedToClass(_) => "Class names can only be words.",
            NoNameAttachedToId(_) => "Id names can only be words.",
            UnclosedCloseBraces(_) => "You have an extra closing brace.",
            UnclosedOpenBraces(_) => "You have an extra open brace.",
            UnexpectedCompDefine(_) => "Attempted to define a component, where it wasn't valid",
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
            ExpectedCompCall(ref lexeme) => lexeme,
            ExpectedVariable(ref lexeme) => lexeme,
            InvalidComponent(ref lexeme) => lexeme,
            InvalidElement(ref lexeme) => lexeme,
            InvalidFunctionCall(ref lexeme) => lexeme,
            InvalidTokenAfterEqualsAttributes(ref lexeme) => lexeme,
            InvalidTokenAfterWordInAttributes(ref lexeme) => lexeme,
            InvalidTokenInAttributes(ref lexeme) => lexeme,
            NoNameAttachedToClass(ref lexeme) => lexeme,
            NoNameAttachedToId(ref lexeme) => lexeme,
            UnclosedCloseBraces(_) => return write!(f, "{}", self.description()),
            UnclosedOpenBraces(_) => return write!(f, "{}", self.description()),
            UnexpectedCompDefine(ref lexeme) => lexeme,
            UnexpectedEof(ref lexeme) => lexeme,
            UnexpectedToken(ref lexeme) => lexeme,
        };
        write!(f, "{}, Got: {}", self.description(), lexeme)
    }
}
