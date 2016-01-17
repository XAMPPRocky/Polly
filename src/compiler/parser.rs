use std::result;
use std::iter::Peekable;
use std::vec::IntoIter;

use super::lexer::Lexer;
use super::tokens::*;
use super::tokens::AstError::*;
use super::tokens::Lexeme::*;
use super::tokens::Operator::*;
use super::tokens::Token::*;

/// Shortens Result<T, AstError> to Result<T>.
pub type AstResult = result::Result<Token, AstError>;

/// The struct detailing the parser itself.
// #[derive(Debug)]
pub struct Parser {
    input: Peekable<IntoIter<Lexeme>>,
    output: Vec<AstResult>,
}

impl Parser {
    /// Generates Parser from Lexer
    pub fn from_lexer(lexer: &Lexer) -> Self {
        let mut parser = Parser {
            input: lexer.output().into_iter().peekable(),
            output: Vec::new(),
        };

        loop {
            match parser.parse_token() {
                Err(Eof) => break,
                token => parser.push(token),
            }
        }
        parser
    }

    fn parse(input: Vec<Lexeme>) -> Self {
        let mut parser = Parser {
            input: input.into_iter().peekable(),
            output: Vec::new(),
        };

        loop {
            match parser.parse_token() {
                Err(Eof) => break,
                token => parser.push(token),
            }
        }

        parser
    }

    fn push(&mut self, token: AstResult) {
        self.output.push(token);
    }

    fn take(&mut self) -> Option<Lexeme> {
        self.input.next()
    }

    fn peek(&mut self) -> Option<Lexeme> {
        match self.input.peek() {
            Some(token) => Some(token.clone()),
            None => None,
        }
    }
    /// Output result vector
    pub fn output(&self) -> Vec<AstResult> {
        // Need to figure out a way to not clone the vector
        self.output.to_vec()
    }

    fn read_leading_quotes(&mut self) -> String {
        let mut value = String::new();
        while let Some(token) = self.take() {
            match token {
                Op(_, Quote) => break,
                Word(_, text) => value.push_str(&*text),
                Op(_, operator) => value.push_str(&*operator.to_string()),
                Empty => {}
            }
        }
        value
    }

    fn parse_token(&mut self) -> AstResult {

        match self.take() {
            // concatenate all the word tokens that are adjacent to each other into a single "Text"
            // token.
            Some(Word(index, word)) => {
                let mut text = String::from(word);
                loop {
                    let peek = self.peek();
                    match peek {
                        Some(Word(_, ref peek_text)) => {
                            text.push_str(&*peek_text);
                            let _ = self.take();
                        }
                        _ => return Ok(Text(text)),
                    }
                }
            }
            // If we find a variable, we expect a word after it.
            Some(Op(index, At)) => {
                match self.take() {
                    Some(Word(index, text)) => {
                        let mut new_text = text.clone();
                        while let Some(Op(_, Dot)) = self.peek() {
                            let _ = self.take();
                            new_text.push('.');

                            match self.take() {
                                Some(Word(_, member)) => new_text.push_str(&*member),
                                Some(unexpected_token) => {
                                    return Err(ExpectedVariable(unexpected_token))
                                }
                                None => return Err(UnexpectedEof(Op(index, Dot))),
                            }
                        }
                        Ok(Variable(new_text))
                    }
                    Some(unexpected_token) => Err(ExpectedVariable(unexpected_token)),
                    None => Err(UnexpectedEof(Op(index, At))),
                }
            }
            // All the operations for creating an element.
            Some(Op(index, ForwardSlash)) => {
                let tag = match self.take() {
                    Some(Word(_, text)) => text,
                    Some(operator) => {
                        return Err(InvalidElement(operator));
                    }
                    None => return Err(UnexpectedEof(Op(index, ForwardSlash))),
                };
                let mut element = Element::new(tag.trim().to_owned());

                'element: while let Some(token) = self.take() {

                    match token {
                        Op(index, At) => {
                            match self.take() {
                                Some(Word(_, id)) => element.add_resource(id),
                                Some(unexpected_token) => {
                                    return Err(ExpectedVariable(unexpected_token))
                                }
                                None => return Err(UnexpectedEof(Op(index, At))),
                            }
                        }
                        Op(index, OpenParam) => {
                            while let Some(token) = self.take() {
                                match token {
                                    Op(_, CloseParam) => {
                                        match self.peek() {
                                            Some(Op(_, OpenBrace)) => break,
                                            _ => return Ok(Html(element)),
                                        }
                                    }
                                    Op(_, Quote) => {
                                        let key = format!("{}{}{}",
                                                          '"',
                                                          self.read_leading_quotes(),
                                                          '"');
                                        element.add_attribute(key, String::from(""));
                                    }
                                    Word(_, key) => {
                                        let value = match self.peek() {
                                            Some(Op(index, Equals)) => {
                                                let _ = self.take();
                                                match self.take() {
                                                    Some(Word(_, text)) => text,
                                                    Some(Op(_, Quote)) => {
                                                        self.read_leading_quotes()
                                                    }
                                                    Some(unexpected_token) => {
                                                        return Err(InvalidTokenInAttributes(unexpected_token));
                                                    }
                                                    None => {
                                                        return Err(UnexpectedEof(Op(index,
                                                                                    Equals)));
                                                    }
                                                }
                                            }
                                            Some(Word(_, _)) => String::from(""),
                                            Some(Op(_, CloseParam)) => String::from(""),
                                            Some(Op(_, Quote)) => String::from(""),
                                            Some(invalid_token) => {
                                                return Err(InvalidTokenInAttributes(invalid_token))
                                            }
                                            None => return Err(UnexpectedEof(Word(index, key))),
                                        };

                                        element.add_attribute(key, value);
                                    }
                                    invalid_token => {
                                        return Err(InvalidTokenInAttributes(invalid_token))
                                    }
                                }
                            }
                        }
                        Op(index, Dot) => {
                            match self.take() {
                                Some(Word(_, class)) => element.add_class(class),
                                Some(unexpected_token) => {
                                    return Err(NoNameAttachedToClass(unexpected_token))
                                }
                                None => return Err(UnexpectedEof(Op(index, Dot))),
                            }
                        }
                        Op(index, Pound) => {
                            match self.take() {
                                Some(Word(_, id)) => element.add_attribute(String::from("id"), id),
                                Some(unexpected_token) => {
                                    return Err(NoNameAttachedToId(unexpected_token))
                                }
                                None => return Err(UnexpectedEof(Op(index, Pound))),
                            }
                        }
                        Op(_, OpenBrace) => {
                            let mut depth: usize = 0;
                            let mut open_brace_index: usize = 0;
                            let mut close_brace_index: usize = 0;
                            let mut children = Vec::new();
                            while let Some(token) = self.take() {
                                match token {
                                    Op(index, OpenBrace) => {
                                        depth += 1;

                                        if depth != 0 {
                                            children.push(Op(index, OpenBrace));
                                        }
                                        open_brace_index = index;
                                    }
                                    Op(index, CloseBrace) => {
                                        if depth == 0 {
                                            break;
                                        } else {
                                            depth -= 1;
                                            children.push(Op(index, CloseBrace));
                                        }
                                        close_brace_index = index;
                                    }
                                    t => children.push(t),
                                }
                            }

                            if depth > 0 {
                                return Err(UnclosedOpenBraces(open_brace_index));
                            } else if depth != 0 {
                                return Err(UnclosedOpenBraces(close_brace_index));
                            }
                            if !children.is_empty() {
                                element.add_children(Parser::parse(children).output());
                            }
                            break;
                        }
                        Word(_, text) => element.add_text(text),

                        unexpected_token => return Err(UnexpectedToken(unexpected_token)),
                    }
                }
                Ok(Html(element))
            }
            Some(Op(index, BackSlash)) => {
                match self.peek() {
                    Some(Op(_, ref operator)) => {
                        let _ = self.take();
                        Ok(Text(operator.to_string()))
                    }
                    Some(_) => Ok(Text(BackSlash.to_string())),
                    None => Err(Eof),
                }
            }
            Some(Op(index, Ampersand)) => unimplemented!(),
            Some(Op(index, operator)) => Ok(Text(operator.to_string())),
            Some(Empty) => unreachable!(),
            None => Err(Eof),
        }
    }
}
