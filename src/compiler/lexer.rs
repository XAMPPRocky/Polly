use std::iter::Peekable;
use std::str::CharIndices;

use super::*;
use super::Lexeme::*;
use super::Operator::*;
/// Lexer
pub struct Lexer<'a> {
    input: Peekable<CharIndices<'a>>,
    output: Vec<Lexeme>,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Lexer {
            input: input.char_indices().peekable(),
            output: Vec::new(),
        }
    }

    fn take(&mut self) -> Option<(usize, char)> {
        self.input.next()
    }

    fn peek(&mut self) -> Option<&(usize, char)> {
        self.input.peek()
    }

    fn push(&mut self, token: Lexeme) {
        self.output.push(token);
    }
    /// Returns the output of the lexer
    pub fn output(&self) -> Vec<Lexeme> {
        let mut new_vec = Vec::new();
        // Need to figure out a way to not clone the vector
        for lexeme in self.output.to_vec() {
            if lexeme != Empty {
                new_vec.push(lexeme);
            }
        }
        new_vec
    }

    /// TODO
    pub fn lex(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);

        while let Some(token) = lexer.take_token() {
            lexer.push(token);
        }

        lexer
    }

    fn take_token(&mut self) -> Option<Lexeme> {
        let mut leading_space = false;
        loop {
            if let Some(&(_, character)) = self.peek() {
                if character.is_whitespace() || character == '\r' {
                    let _ = self.take();
                    leading_space = true;
                } else {
                    break;
                }
            } else {
                return None;
            }
        }

        match self.take() {
            Some((index, AMPERSAND)) => Some(Symbol(index, Ampersand)),
            Some((index, AT)) => Some(Symbol(index, At)),
            Some((index, BACKSLASH)) => Some(Symbol(index, BackSlash)),
            Some((index, COMMA)) => Some(Symbol(index, Comma)),
            Some((index, CLOSEBRACE)) => Some(Symbol(index, CloseBrace)),
            Some((index, CLOSEPARAM)) => Some(Symbol(index, CloseParam)),
            Some((index, DOLLAR)) => Some(Symbol(index, Dollar)),
            Some((index, DOT)) => Some(Symbol(index, Dot)),
            Some((index, DOUBLEQUOTE)) => Some(Symbol(index, Quote)),
            Some((index, EQUALS)) => Some(Symbol(index, Equals)),
            Some((index, FORWARDSLASH)) => Some(Symbol(index, ForwardSlash)),
            Some((index, OPENBRACE)) => Some(Symbol(index, OpenBrace)),
            Some((index, OPENPARAM)) => Some(Symbol(index, OpenParam)),
            Some((index, POUND)) => Some(Symbol(index, Pound)),
            Some((index, STAR)) => Some(Symbol(index, Star)),
            Some((index, character)) => {
                let mut word = if leading_space {
                    ' '.to_string()
                } else {
                    String::new()
                };

                word.push(character);

                loop {
                    if let Some(&(_, character)) = self.peek() {
                        match character {
                            // The following case is for determining if a character divides words or
                            // if it is packaged with the words. So things like "Hello}" comes out
                            // as Text: "Hello" Operator: "}"
                            AMPERSAND |
                            AT |
                            BACKSLASH |
                            CLOSEBRACE |
                            CLOSEPARAM |
                            DOLLAR |
                            DOT |
                            DOUBLEQUOTE |
                            EQUALS |
                            FORWARDSLASH |
                            OPENBRACE |
                            OPENPARAM |
                            POUND |
                            SINGLEQUOTE |
                            CARRAGE_RETURN => {
                                return Some(Word(index, word));
                            }
                            ch => {
                                if !ch.is_whitespace() {
                                    word.push(self.take().unwrap().1);
                                } else {
                                    break;
                                }
                            }
                        }
                    } else {
                        break;
                    }
                }

                if let Some(&(_, ch)) = self.peek() {
                    if ch.is_whitespace() {
                        word.push(ch);
                        let _ = self.take();
                    }
                }
                Some(Word(index, word))
            }
            None => None,
        }
    }
}

#[allow(unused_imports)]
mod tests {
    use super::Lexer;
    use compiler::tokens::Lexeme;
    use compiler::tokens::Lexeme::{Word, Symbol};
    use compiler::tokens::Operator::*;

    #[test]
    fn ignore_spaces() {
        let lexer = Lexer::lex(" \t   ");

        assert_eq!(lexer.output(), vec![]);
    }

    #[test]
    fn ampersand_operator() {
        let lexer = Lexer::lex("&");

        assert_eq!(lexer.output(), vec![Symbol(0, Ampersand)]);
    }

    #[test]
    fn at_operator() {
        let lexer = Lexer::lex("@");

        assert_eq!(lexer.output(), vec![Symbol(0, At)]);
    }
    #[test]
    fn back_slash_operator() {
        let lexer = Lexer::lex("\\");

        assert_eq!(lexer.output(), vec![Symbol(0, BackSlash)]);
    }
    #[test]
    fn close_brace_operator() {
        let lexer = Lexer::lex("}");

        assert_eq!(lexer.output(), vec![Symbol(0, CloseBrace)]);
    }
    #[test]
    fn close_param_operator() {
        let lexer = Lexer::lex(")");

        assert_eq!(lexer.output(), vec![Symbol(0, CloseParam)]);
    }
    #[test]
    fn dollar_operator() {
        let lexer = Lexer::lex("$");

        assert_eq!(lexer.output(), vec![Symbol(0, Dollar)]);
    }
    #[test]
    fn dot_operator() {
        let lexer = Lexer::lex(".");

        assert_eq!(lexer.output(), vec![Symbol(0, Dot)]);
    }
    #[test]
    fn equals_operator() {
        let lexer = Lexer::lex("=");

        assert_eq!(lexer.output(), vec![Symbol(0, Equals)]);
    }
    #[test]
    fn forward_slash_operator() {
        let lexer = Lexer::lex("/");

        assert_eq!(lexer.output(), vec![Symbol(0, ForwardSlash)]);
    }
    #[test]
    fn open_brace_operator() {
        let lexer = Lexer::lex("{");

        assert_eq!(lexer.output(), vec![Symbol(0, OpenBrace)]);
    }
    #[test]
    fn open_param_operator() {
        let lexer = Lexer::lex("(");

        assert_eq!(lexer.output(), vec![Symbol(0, OpenParam)]);
    }
    #[test]
    fn pound_operator() {
        let lexer = Lexer::lex("#");

        assert_eq!(lexer.output(), vec![Symbol(0, Pound)]);
    }
    #[test]
    fn quote_operator() {
        let lexer = Lexer::lex("\"");

        assert_eq!(lexer.output(), vec![Symbol(0, Quote)]);
    }
    #[test]
    fn star_operator() {
        let lexer = Lexer::lex("*");

        assert_eq!(lexer.output(), vec![Symbol(0, Star)]);
    }
    #[test]
    fn all_operators() {
        let lexer = Lexer::lex("&@\\})$.=/{(#\"*,");
        let expected = vec![Symbol(0, Ampersand),
                            Symbol(1, At),
                            Symbol(2, BackSlash),
                            Symbol(3, CloseBrace),
                            Symbol(4, CloseParam),
                            Symbol(5, Dollar),
                            Symbol(6, Dot),
                            Symbol(7, Equals),
                            Symbol(8, ForwardSlash),
                            Symbol(9, OpenBrace),
                            Symbol(10, OpenParam),
                            Symbol(11, Pound),
                            Symbol(12, Quote),
                            Symbol(13, Star),
                            Symbol(14, Comma)];

        for (actual, expected) in lexer.output().iter().zip(expected.iter()) {
            assert_eq!(actual, expected);
        }
    }
    #[test]
    fn word() {
        let lexer = Lexer::lex("Hello");

        assert_eq!(lexer.output(), vec![Word(0, "Hello".to_owned())]);
    }
    #[test]
    fn words() {
        let lexer = Lexer::lex("The Lord Of The Rings");

        assert_eq!(lexer.output(),
                   vec![Word(0, "The ".to_owned()),
                        Word(4, "Lord ".to_owned()),
                        Word(9, "Of ".to_owned()),
                        Word(12, "The ".to_owned()),
                        Word(16, "Rings".to_owned())]);
    }
    #[test]
    fn words_and_operators() {
        let lexer = Lexer::lex("@{Hello}.");

        assert_eq!(lexer.output(),
                   vec![Symbol(0, At),
                        Symbol(1, OpenBrace),
                        Word(2, "Hello".to_string()),
                        Symbol(7, CloseBrace),
                        Symbol(8, Dot)]);
    }
    #[test]
    fn hello_world() {
        let lexer = Lexer::lex("/html{ /body { /p{Hello /u{World}!}}}");
        let expected_tokens = vec![Symbol(0, ForwardSlash),
                                   Word(1, "html".to_owned()),
                                   Symbol(5, OpenBrace),
                                   Symbol(7, ForwardSlash),
                                   Word(8, "body ".to_owned()),
                                   Symbol(13, OpenBrace),
                                   Symbol(15, ForwardSlash),
                                   Word(16, "p".to_owned()),
                                   Symbol(17, OpenBrace),
                                   Word(18, "Hello ".to_owned()),
                                   Symbol(24, ForwardSlash),
                                   Word(25, "u".to_owned()),
                                   Symbol(26, OpenBrace),
                                   Word(27, "World".to_owned()),
                                   Symbol(32, CloseBrace),
                                   Word(33, "!".to_owned()),
                                   Symbol(34, CloseBrace),
                                   Symbol(35, CloseBrace),
                                   Symbol(36, CloseBrace)];
        for (actual, expected) in lexer.output().iter().zip(expected_tokens.iter()) {
            assert_eq!(actual, expected);
        }
    }
}
