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
            Some((index, AMPERSAND)) => Some(Op(index, Ampersand)),
            Some((index, AT)) => Some(Op(index, At)),
            Some((index, BACKSLASH)) => Some(Op(index, BackSlash)),
            Some((index, CLOSEBRACE)) => Some(Op(index, CloseBrace)),
            Some((index, CLOSEPARAM)) => Some(Op(index, CloseParam)),
            Some((index, DOLLAR)) => Some(Op(index, Dollar)),
            Some((index, DOT)) => Some(Op(index, Dot)),
            Some((index, DOUBLEQUOTE)) => Some(Op(index, Quote)),
            Some((index, EQUALS)) => Some(Op(index, Equals)),
            Some((index, FORWARDSLASH)) => Some(Op(index, ForwardSlash)),
            Some((index, OPENBRACE)) => Some(Op(index, OpenBrace)),
            Some((index, OPENPARAM)) => Some(Op(index, OpenParam)),
            Some((index, POUND)) => Some(Op(index, Pound)),
            Some((index, STAR)) => Some(Op(index, Star)),
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
                            SINGLEQUOTE => {
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
    use compiler::tokens::Lexeme::{Word, Op};
    use compiler::tokens::Operator::*;

    #[test]
    fn ignore_spaces() {
        let lexer = Lexer::lex(" \t   ");

        assert_eq!(lexer.output(), vec![]);
    }

    #[test]
    fn at_operator() {
        let lexer = Lexer::lex("@");

        assert_eq!(lexer.output(), vec![Op(0, At)]);
    }
    #[test]
    fn back_slash_operator() {
        let lexer = Lexer::lex("\\");

        assert_eq!(lexer.output(), vec![Op(0, BackSlash)]);
    }
    #[test]
    fn close_brace_operator() {
        let lexer = Lexer::lex("}");

        assert_eq!(lexer.output(), vec![Op(0, CloseBrace)]);
    }
    #[test]
    fn close_param_operator() {
        let lexer = Lexer::lex(")");

        assert_eq!(lexer.output(), vec![Op(0, CloseParam)]);
    }
    #[test]
    fn dot_operator() {
        let lexer = Lexer::lex(".");

        assert_eq!(lexer.output(), vec![Op(0, Dot)]);
    }
    #[test]
    fn equals_operator() {
        let lexer = Lexer::lex("=");

        assert_eq!(lexer.output(), vec![Op(0, Equals)]);
    }
    #[test]
    fn forward_slash_operator() {
        let lexer = Lexer::lex("/");

        assert_eq!(lexer.output(), vec![Op(0, ForwardSlash)]);
    }
    #[test]
    fn new_line_operator() {
        let lexer = Lexer::lex("\n");

        assert_eq!(lexer.output(), vec![Op(0, Newline)]);
    }
    #[test]
    fn open_brace_operator() {
        let lexer = Lexer::lex("{");

        assert_eq!(lexer.output(), vec![Op(0, OpenBrace)]);
    }
    #[test]
    fn open_param_operator() {
        let lexer = Lexer::lex("(");

        assert_eq!(lexer.output(), vec![Op(0, OpenParam)]);
    }
    #[test]
    fn pound_operator() {
        let lexer = Lexer::lex("#");

        assert_eq!(lexer.output(), vec![Op(0, Pound)]);
    }
    #[test]
    fn quote_operator() {
        let lexer = Lexer::lex("\"");

        assert_eq!(lexer.output(), vec![Op(0, Quote)]);
    }
    #[test]
    fn single_quote_into_quote_operator() {
        let lexer = Lexer::lex("'");

        assert_eq!(lexer.output(), vec![Op(0, Quote)]);
    }
    #[test]
    fn star_operator() {
        let lexer = Lexer::lex("*");

        assert_eq!(lexer.output(), vec![Op(0, Star)]);
    }
    #[test]
    fn all_operators() {
        let lexer = Lexer::lex("&@\\})$.=/\n{(#\"'*");

        assert_eq!(lexer.output(),
                   vec![Op(0, Ampersand),
                        Op(1, At),
                        Op(2, BackSlash),
                        Op(3, CloseBrace),
                        Op(4, CloseParam),
                        Op(5, Dollar),
                        Op(6, Dot),
                        Op(7, Equals),
                        Op(8, ForwardSlash),
                        Op(9, Newline),
                        Op(10, OpenBrace),
                        Op(11, OpenParam),
                        Op(12, Pound),
                        Op(13, Quote),
                        Op(14, Quote),
                        Op(15, Star)]);
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
                   vec![Op(0, At),
                        Op(1, OpenBrace),
                        Word(2, "Hello".to_string()),
                        Op(7, CloseBrace),
                        Op(8, Dot)]);
    }
    #[test]
    fn hello_world() {
        let lexer = Lexer::lex("/html{\n /body {\n /p{Hello /u{World}!}}}");
        let expected_tokens = vec![Op(0, ForwardSlash),
                                   Word(1, "html".to_owned()),
                                   Op(5, OpenBrace),
                                   Op(6, Newline),
                                   Op(8, ForwardSlash),
                                   Word(9, "body ".to_owned()),
                                   Op(14, OpenBrace),
                                   Op(15, Newline),
                                   Op(17, ForwardSlash),
                                   Word(18, "p".to_owned()),
                                   Op(19, OpenBrace),
                                   Word(20, "Hello ".to_owned()),
                                   Op(26, ForwardSlash),
                                   Word(27, "u".to_owned()),
                                   Op(28, OpenBrace),
                                   Word(29, "World".to_owned()),
                                   Op(34, CloseBrace),
                                   Word(35, "!".to_owned()),
                                   Op(36, CloseBrace),
                                   Op(37, CloseBrace),
                                   Op(38, CloseBrace)];
        for (actual, expected) in lexer.output().iter().zip(expected_tokens.iter()) {
            assert_eq!(actual, expected);
        }
    }
}
