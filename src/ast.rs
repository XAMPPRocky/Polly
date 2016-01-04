use element::Element;

/// TODO
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// TODO
    Html(usize, Element),
    /// TODO
    Text(usize, String),
    /// TODO
    Variable(usize, String),
    /// Newline
    Endofline,
    /// TODO
    Blank,
    /// TODO
    Comment(usize, String),
}

/// Errors defining all the errors that can be encountered while parsing.
#[derive(Debug, PartialEq, Clone)]
pub enum AstError {
    /// End of File
    Eof,
    /// Expected a Variable name.
    ExpectedVariable(usize),
    /// No name attached to element.
    InvalidToken(usize),
    /// Token that isn't (, ), =, ", ', or a word. 
    InvalidTokenInAttributes(usize),
    /// Having a . without anything following it up.
    NoNameAttachedToClass(usize),
    /// Having a # without anything following it up.
    NoNameAttachedToId(usize),
    /// Extra } braces
    UnclosedCloseBraces(usize),
    /// Extra { braces
    UnclosedOpenBraces(usize),
    /// File ended while we tried to parse element.
    UnexpectedEof,
    /// Unknown token
    UnexpectedToken(usize),
}
