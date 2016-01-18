use std::collections::HashMap;
use std::result;
use std::process;

use super::*;

pub struct Codegen<'a> {
    elements: Vec<AstResult>,
    source: &'a str,
    file: &'a str,
    symbol_map: HashMap<&'a str, &'a str>,
}

impl<'a> Codegen<'a> {
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

    pub fn from_component(component: &Component) -> String {
        unimplemented!()
    }

    fn render(&self, token: &AstResult) -> Option<String> {
        use super::tokens::Token::*;
        match token {
            &Ok(Html(ref element)) => {
                use std::io::Write;
                let mut html = Vec::new();
                let tag = &**element.tag();
                write!(&mut html, "<{tag}", tag = tag).ok().expect("Couldn't write to html file");

                if !element.classes().is_empty() {
                    write!(&mut html, " class=\"").ok().expect("Couldn't write to html file");
                    let mut classes_iter = element.classes().iter();
                    write!(&mut html, "{}", classes_iter.next().unwrap())
                        .ok()
                        .expect("Couldn't write to html file");
                    for class in classes_iter {
                        if !class.is_empty() {
                            write!(&mut html, " {}", &*class)
                                .ok()
                                .expect("Couldn't write to html file");
                        }
                    }
                    write!(&mut html, "\"").ok().expect("Couldn't write to html file");
                }

                if !element.attributes().is_empty() {
                    for (key, value) in element.attributes() {
                        if !key.is_empty() {
                            if !value.is_empty() {
                                write!(&mut html, " {key}=\"{value}\"", key = key, value = value)
                                    .ok()
                                    .expect("Couldn't write to html file");
                            } else {
                                write!(&mut html, " {}", key)
                                    .ok()
                                    .expect("Couldn't write to html file");
                            }
                        }
                    }
                }

                write!(&mut html, ">").ok().expect("Couldn't write to html file");


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
                            write!(&mut html, "{}", Codegen::codegen(value, &*resource))
                                .ok()
                                .expect("Couldn't write to html file");
                        }
                        None => (),
                    }
                } else {
                    for child in element.children() {
                        if let Some(rendered_child) = self.render(child) {
                            write!(&mut html, "{}", rendered_child)
                                .ok()
                                .expect("Couldn't write to html file");
                        }
                    }
                }

                write!(&mut html, "</{tag}>", tag = tag).ok().expect("Couldn't write to html file");

                Some(String::from_utf8(html).unwrap())
            }
            &Ok(Text(ref text)) => Some(text.clone()),
            &Ok(Variable(ref variable)) => {
                match self.symbol_map.get(&**variable) {
                    Some(value) => Some(String::from(*value)),
                    None => Some("".to_owned()),
                }
            }
            &Ok(Comp(ref ast)) => unimplemented!(),
            &Ok(Function(ref function)) => unimplemented!(),
            &Err(ref error) => {
                use super::tokens::AstError::*;
                let (index, token_length) = match *error {
                    Eof => return None,
                    ExpectedVariable(ref lexeme) => (lexeme.index(), lexeme.length()),
                    InvalidElement(ref lexeme) => (lexeme.index(), lexeme.length()),
                    InvalidTokenAfterEqualsAttributes(ref lexeme) => {
                        (lexeme.index(), lexeme.length())
                    }
                    InvalidTokenAfterWordInAttributes(ref lexeme) => {
                        (lexeme.index(), lexeme.length())
                    }
                    InvalidTokenInAttributes(ref lexeme) => (lexeme.index(), lexeme.length()),
                    NoNameAttachedToClass(ref lexeme) => (lexeme.index(), lexeme.length()),
                    NoNameAttachedToId(ref lexeme) => (lexeme.index(), lexeme.length()),
                    UnclosedCloseBraces(index) => (index, 1),
                    UnclosedOpenBraces(index) => (index, 1),
                    UnexpectedEof(ref lexeme) => (lexeme.index(), lexeme.length()),
                    UnexpectedToken(ref lexeme) => (lexeme.index(), lexeme.length()),
                };
                let mut line_number: usize = 0;
                let mut col_number: usize = 1;

                for ch in self.source[..index].chars() {
                    col_number += 1;
                    if ch == '\n' {
                        line_number += 1;
                        col_number = 1;
                    }
                }

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
                println!("");
                let mut underline = String::from("^");

                for _ in 1..token_length {
                    underline.push('~');
                }
                let file_name_print = format!("{}:{}:{}:", self.file, line_number, col_number);
                println!("{} {}", file_name_print, error);
                println!("{} {}", file_name_print, section.trim());
                println!("{0:>1$}",
                         underline,
                         col_number + 3 + file_name_print.len() -
                         (section.len() - section.trim().len()));
                process::exit(0);
            }
        }
    }
}


mod tests {

    #[test]
    fn test_codegen() {
        use super::Codegen;
        use std::fs::File;
        use std::io::Read;
        let file_name = "./tests/test.poly";
        let mut file = File::open(file_name)
                           .ok()
                           .expect("File doesn't exist, or isn't a file.");
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).ok().expect("File contents corrupted");

        let html = Codegen::codegen(&*file_contents, file_name);
        assert!(!html.is_empty());
    }
}
