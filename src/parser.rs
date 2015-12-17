use std::collections::HashMap;
use token::Token;

pub struct Parser<'a> {
    input: Vec<char>,
    position: usize,
    line: usize,
    output: Vec<Token<'a>>,
    symbol_table: HashMap<&'a str, &'a str>,
}

pub enum ParseError {
    eof,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser {
            input: input.chars().collect(),
            position: 0,
            line: 0,
            output: Vec::new(),
            symbol_table: HashMap::new(),
        }
    }

    fn take(&mut self) -> Option<char> {
        if self.eof() {
            None
        } else {
            let ch = self.input[self.position];
            self.position += 1;

            if ch == '\n' {
                self.line += 1;
            }

            Some(ch)
        }
    }
    fn peek(&mut self) -> Option<char> {
        if self.eof() {
            None
        } else {
            Some(self.input[self.position])
        }
    }

    fn eof(&self) -> bool {
        self.position >= self.input.len()
    }

    fn take_until(&mut self, character: char, section: &mut String) -> Result<(), ParseError> {

        while let Some(ch) = self.take() {
            if ch == character {
                return Ok(());
            } else {
                section.push(ch)
            }
        }

        Err(ParseError::eof)
    }

    fn peek_until(&mut self, character: char, section: &mut String) -> Result<(), ParseError> {
        let fake_position = self.position;

        while fake_position >= self.position {
            let ch = self.input[fake_position];
            if ch == character {
                return Ok(());
            } else {
                section.push(ch);
            }
        }

        Err(ParseError::eof)
    }
    fn take_while<F>(&mut self, section: &mut String, condition: F) -> Result<(), ParseError>
        where F: Fn(char) -> bool
    {
        while let Some(ch) = self.take() {
            if condition(ch) {
                section.push(ch);
            } else {
                break;
            }
        }

        if let Some(_) = self.peek() {
            Ok(())
        } else {
            Err(ParseError::eof)
        }
    }

    fn parse(&mut self) {
        unimplemented!()
    }
}
