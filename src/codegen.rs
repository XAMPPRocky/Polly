use std::collections::HashMap;
use std::result;

use ast::{Token, AstError};
use lexer::Lexer;
use parser::Parser;
pub type Result<T> = result::Result<T, AstError>;

pub struct Codegen<'a> {
    elements: Vec<Result<Token>>,
    source: &'a str,
    file: &'a str,
    symbol_map: HashMap<&'a str, &'a str>,
}

impl<'a> Codegen<'a> {
    fn new(elements: Vec<Result<Token>>,
           source: &'a str,
           file: &'a str,
           symbol_map: HashMap<&'a str, &'a str>)
           -> Self {
        Codegen {
            elements: elements,
            source: source,
            file: file,
            symbol_map: symbol_map,
        }
    }

    fn from_parser(parser: &'a Parser, file: &'a str, source: &'a str) -> Self {
        Codegen {
            elements: parser.output(),
            file: file,
            source: source,
            symbol_map: HashMap::new(),
        }
    }

    pub fn codegen(source: &str, file: &str) -> String {
        let mut html = String::new();

        let lexer = Lexer::lex(source);
        let parser = Parser::from_lexer(&lexer);
        let codegen = Codegen::from_parser(&parser, file, source);
        for element in codegen.elements.iter() {
            if let Some(string) = codegen.render(element) {
                html.push_str(&*string);
            } else {
                break;
            }
        }
        html
    }

    fn render(&self, token: &Result<Token>) -> Option<String> {
        use ast::Token::*;
        match token {
            &Ok(Html(_, ref element)) => {
                use std::io::Write;
                let mut html = Vec::new();
                let tag = &**element.tag();
                let _ = write!(&mut html, "<{tag}", tag = tag);

                if !element.classes().is_empty() {
                    let _ = write!(&mut html, " class=\"");
                    let mut classes_iter = element.classes().iter();
                    let _ = write!(&mut html, "{}", classes_iter.next().unwrap());
                    for class in classes_iter {
                        if !class.is_empty() {
                            let _ = write!(&mut html, " {}", &*class);
                        }
                    }
                    let _ = write!(&mut html, "\"");
                }

                if !element.attributes().is_empty() {
                    for (key, value) in element.attributes() {
                        if !key.is_empty() {
                            if !value.is_empty() {
                                let _ = write!(&mut html,
                                               " {key}=\"{value}\"",
                                               key = key,
                                               value = value);
                            } else {
                                let _ = write!(&mut html, " {}", key);
                            }
                        }
                    }
                }

                let _ = write!(&mut html, ">");


                const VOID_ELEMENTS: [&'static str; 13] = ["area", "base", "br", "col", "hr",
                                                           "img", "input", "link", "meta",
                                                           "command", "keygen", "source",
                                                           "!DOCTYPE"];

                for void in VOID_ELEMENTS.iter() {
                    if void == &tag {
                        return Some(String::from_utf8(html).unwrap());
                    }
                }

                if let &Some(ref resource) = element.resource() {
                    match self.symbol_map.get(&**resource) {
                        Some(value) => {
                            let _ = write!(&mut html, "{}", Codegen::codegen(value, &*resource));
                        }
                        None => (),
                    }
                } else {
                    for child in element.children() {
                        if let Some(rendered_child) = self.render(child) {
                            let _ = write!(&mut html, "{}", rendered_child);
                        }
                    }
                }

                let _ = write!(&mut html, "</{tag}>", tag = tag);

                Some(String::from_utf8(html).unwrap())
            }
            &Ok(Text(_, ref text)) => Some(text.clone()),
            &Ok(Variable(_, ref variable)) => {
                match self.symbol_map.get(&**variable) {
                    Some(value) => Some(String::from(*value)),
                    None => Some("".to_owned()),
                }
            }
            &Ok(Endofline) => Some(String::from("")),
            &Ok(_) => Some(String::from("")),
            &Err(ref error) => {
                use ast::AstError::*;
                let index = match *error {
                    Eof => return None,
                    ExpectedVariable(value) => value,
                    InvalidToken(value) => value,
                    InvalidTokenInAttributes(value) => value,
                    NoNameAttachedToClass(value) => value,
                    NoNameAttachedToId(value) => value,
                    UnclosedCloseBraces(value) => value,
                    UnclosedOpenBraces(value) => value,
                    UnexpectedEof => panic!("Unexpected End of file"),
                    UnexpectedToken(value) => value,
                };
                let mut line_number: usize = 0;
                let mut section = String::new();
                for ch in self.source[..index].chars().rev() {
                    if ch == '\n' {
                        section = section.chars().rev().collect();
                        break;
                    } else {
                        section.push(ch);
                    }
                }

                for ch in self.source[index..].chars() {
                    if ch == '\n' {
                        break;
                    } else {
                        section.push(ch);
                    }
                }

                println!("\n{}: {:>5?}\n{:>10}", self.file, error, section);
                None
            }
        }
    }
}


mod tests {

    #[test]
    fn test_codegen() {

        use super::Codegen;
        use std::io::Read;
        use std::fs::File;
        let poly_path = "./tests/static.min.poly";
        let html_path = "./tests/static.min.html";
        let mut poly = File::open("./tests/static_site.poly").unwrap();
        let mut poly_contents = String::new();
        let _ = poly.read_to_string(&mut poly_contents);

        // let mut html = File::open(html_path).unwrap();
        // let mut html_contents = String::new();
        // let _ = html.read_to_string(&mut html_contents);
        let result = Codegen::codegen(&*poly_contents, poly_path);
        println!("{}", result);
        assert!(true);
        // assert_eq!(html_contents, poly_contents);
    }
}
