use std::result;
use std::iter::Peekable;
use lexer::{Lexer, Lexeme};
use lexer::Lexeme::*;
use operator::Operator::*;
use element::Element;
use ast::{Token, AstError};
use ast::Token::*;
use ast::AstError::*;
use std::vec::IntoIter;

/// Shortens Result<T, AstError> to Result<T>.
pub type Result<T> = result::Result<T, AstError>;

/// The struct detailing the parser itself.
// #[derive(Debug)]
pub struct Parser {
    input: Peekable<IntoIter<Lexeme>>,
    output: Vec<Result<Token>>,
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

    fn push(&mut self, token: Result<Token>) {
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
    pub fn output(&self) -> Vec<Result<Token>> {
        self.output.to_vec()
    }

    fn parse_token(&mut self) -> Result<Token> {

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
                        _ => return Ok(Text(index, text)),
                    }
                }
            }
            // If we find a variable, we expect a word after it.
            Some(Operator(index, At)) => {
                match self.take() {
                    Some(Word(_, text)) => Ok(Variable(index, text)),
                    _ => Err(ExpectedVariable(index)),
                }
            }
            // All the operations for creating an element.
            Some(Operator(index, ForwardSlash)) => {
                // checking if the forward slash was used for a comment.
                match self.peek() {
                    Some(Operator(index, ref op @ ForwardSlash)) |
                    Some(Operator(index, ref op @ Star)) => {
                        let _ = self.take();
                        let is_single = if *op == ForwardSlash {
                            true
                        } else {
                            false
                        };
                        let mut comment = String::new();
                        while let Some(token) = self.take() {
                            match token {
                                Operator(_, Newline) if is_single => break,
                                Operator(_, Star) if !is_single => {
                                    if let Some(token) = self.take() {
                                        match token {
                                            Operator(_, ForwardSlash) => break,
                                            Operator(_, op) => comment.push_str(&*op.to_string()),
                                            Word(_, text) => comment.push_str(&*text),
                                        }
                                    }
                                }
                                Operator(_, op) => comment.push_str(&*op.to_string()),
                                Word(_, text) => comment.push_str(&*text),
                            }
                        }
                        return Ok(Comment(index, comment));
                    }
                    Some(_) => {}
                    _ => return Err(Eof),
                }

                let tag = match self.take() {
                    Some(Word(_, text)) => text,
                    _ => return Err(InvalidToken(index)),
                };
                let mut element = Element::new(tag.trim().to_owned());

                'element: while let Some(token) = self.take() {

                    match token {
                        Operator(index, At) => {
                            match self.take() {
                                Some(Word(_, id)) => element.add_resource(id),
                                _ => return Err(ExpectedVariable(index)),
                            }
                        }
                        Operator(index, OpenParam) => {
                            while let Some(token) = self.take() {
                                match token {
                                    Operator(_, CloseParam) => {
                                        match self.peek() {
                                            Some(Operator(_, OpenBrace)) => break,
                                            _ => return Ok(Html(index, element)),
                                        }
                                    }
                                    Word(_, key) => {
                                        let value = match self.peek() {
                                            Some(Operator(index, Equals)) => {
                                                let _ = self.take();
                                                match self.take() {
                                                    Some(Word(_, text)) => text,
                                                    Some(Operator(_, Quote)) => {
                                                        let mut value = String::new();
                                                        while let Some(token) = self.take() {
                                                            match token {
                                                                Operator(_, Quote) => break,
                                                                Word(_, text) => {
                                                                    value.push_str(&*text)
                                                                }
                                                                Operator(_, operator) => {
                                                                    value.push_str(&*operator.to_string())
                                                                }
                                                            }
                                                        }
                                                        value
                                                    }
                                                    Some(_) => {
                                                        return Err(InvalidTokenInAttributes(index));
                                                    }
                                                    None => return Err(UnexpectedEof),
                                                }
                                            }
                                            Some(Word(_, _)) => String::from(""),
                                            Some(Operator(_, CloseParam)) => String::from(""),
                                            _ => return Err(InvalidTokenInAttributes(index)),
                                        };

                                        element.add_attribute(key, value);
                                    }
                                    _ => return Err(InvalidTokenInAttributes(index)),
                                }
                            }
                        }
                        Operator(index, Dot) => {
                            match self.take() {
                                Some(Word(_, class)) => element.add_class(class),
                                _ => return Err(NoNameAttachedToClass(index)),
                            }
                        }
                        Operator(index, Pound) => {
                            match self.take() {
                                Some(Word(_, id)) => element.add_attribute(String::from("id"), id),
                                _ => return Err(NoNameAttachedToId(index)),
                            }
                        }
                        Operator(_, OpenBrace) => {
                            let mut depth: usize = 0;
                            let mut open_brace_index: usize = 0;
                            let mut close_brace_index: usize = 0;
                            let mut children = Vec::new();
                            while let Some(token) = self.take() {
                                match token {
                                    Operator(index, OpenBrace) => {
                                        depth += 1;

                                        if depth != 0 {
                                            children.push(Operator(index, OpenBrace));
                                        }
                                        open_brace_index = index;
                                    }
                                    Operator(index, CloseBrace) => {
                                        if depth == 0 {
                                            break;
                                        } else {
                                            depth -= 1;
                                            children.push(Operator(index, CloseBrace));
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
                        Word(index, text) => element.add_text(index, text),

                        _ => return Err(UnexpectedToken(index)),
                    }
                }
                Ok(Html(index, element))
            }
            Some(Operator(index, BackSlash)) => {
                match self.peek() {
                    Some(Operator(_, ref operator)) => {
                        let _ = self.take();
                        Ok(Text(index, operator.to_string()))
                    }
                    Some(_) => Ok(Text(index, BackSlash.to_string())),
                    None => Err(Eof),
                }
            }
            Some(Operator(_, Newline)) => Ok(Endofline),
            // Some(Operator(index, Quote)) => {
            //     let mut section = String::new();
            //
            //     loop {
            //
            //         match self.peek() {
            //             Some(Operator(_, Quote)) => break,
            //             Some(Operator(_, operator)) => section.push_str(&*operator.to_string()),
            //             Some(Word(_, word)) => section.push_str(&*word),
            //             None => return Err(UnexpectedEof),
            //         };
            //     }
            //     Ok(Text(index, section))
            // }
            Some(Operator(index, operator)) => Ok(Text(index, operator.to_string())),
            None => Err(Eof),
        }
    }
}

mod tests {
    use super::*;
    use lexer::*;
    #[test]
    fn parse() {
        use std::io::Read;
        use std::fs::File;
        let file_path = "./tests/static_site.poly";
        let mut file = File::open(file_path).unwrap();
        let mut contents = String::new();
        let _ = file.read_to_string(&mut contents);
        // println!("{:?}", contents);
        let lexer = Lexer::lex(&*contents);
        let parser = Parser::from_lexer(&lexer);

        // println!("{:#?}", parser.output());
        assert!(true);
    }
}
