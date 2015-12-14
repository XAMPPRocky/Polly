use std::collections::HashMap;
use token::token::Token;
#[derive(Debug)]
struct Parser<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    output: Vec<Box<Token + 'a>>,
    symbol_table: HashMap<&'a str, &'a str>,
}


impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser {
            input: input,
            position: 0,
            line: 0,
            output: Vec::new(),
            symbol_table: HashMap::new(),
        }
    }

    fn parse(&mut self) {
        // add code here
    }
}
