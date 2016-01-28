use std::collections::BTreeMap;
use std::process;

use serde_json::Value;
use super::*;
use template::GlobalComponents;

macro_rules! exit {
    () => {{
      if cfg!(debug_assertions) {
          panic!()
      } else {
          process::exit(101);
      }  
    }}
}
#[derive(Debug, Clone)]
pub struct Codegen<'a> {
    elements: Vec<AstResult>,
    source: &'a str,
    file: &'a str,
    variables: BTreeMap<String, Value>,
}

impl<'a> Codegen<'a> {
    fn from_parser(parser: &'a Parser, file: &'a str, source: &'a str, json: Value) -> Self {
        if let Value::Object(object) = json {
            Codegen {
                elements: parser.output(),
                file: file,
                source: source,
                variables: object,
            }
        } else {
            println!("JSON wasn't valid. JSON: {:?}", json);
            exit!();
        }
    }

    pub fn codegen(source: &str, file: &str, json: Value) -> String {
        let mut html = String::new();

        let lexer = Lexer::lex(source);
        let parser = Parser::from_lexer(&lexer);
        let codegen = Codegen::from_parser(&parser, file, source, json);
        for element in codegen.elements.iter() {
            if let Some(string) = codegen.render(element) {
                html.push_str(&*string);
            } else {
                break;
            }
        }
        html
    }

    fn from_component(&self, component_call: ComponentCall) -> String {
        if let Some(component) = GlobalComponents::unlock().get(&*component_call.name()) {
            let args = component.args();
            let values = component_call.values();
            let mut arg_map: BTreeMap<String, Value> = BTreeMap::new();
            println!("{:?}", (args.len(), values.len()));
            if args.len() == values.len() {
                let zipped = args.iter().zip(values.iter());

                for (ref arg, ref value) in zipped {

                    match *arg {
                        &Args::Text(ref arg_name) => {
                            let &Args::Text(ref arg_value) = *value;
                            let value = match self.variables.get(arg_value) {
                                Some(text) => text.clone(),
                                None => Value::Null,
                            };
                            arg_map.insert(arg_name.clone(),
                                           Value::String(value_to_string(&value)));
                        }
                    }
                }
                let codegen = Codegen { elements: component.ast(), ..self.clone() };
                let mut html = String::new();
                for element in codegen.elements.iter() {
                    if let Some(string) = codegen.render(element) {
                        html.push_str(&*string);
                    } else {
                        break;
                    }
                }
                html
            } else {
                println!("Incorrect number of arguments passed");
                exit!()
            }
        } else {
            println!("Component {} doesn't exist make sure it was imported correctly",
                     component_call.name());
            exit!()
        }
    }

    fn render(&self, token: &AstResult) -> Option<String> {
        use super::Token::*;
        match token {
            &Ok(Html(ref element)) => {
                use std::io::Write;
                let mut html = Vec::new();
                let tag = &**element.tag();
                const HTML_ERROR: &'static str = "Couldn't write to html buffer.";
                write!(&mut html, "<{tag}", tag = tag).ok().expect(HTML_ERROR);

                if !element.classes().is_empty() {
                    write!(&mut html, " class=\"").ok().expect(HTML_ERROR);
                    let mut classes_iter = element.classes().iter();
                    write!(&mut html, "{}", classes_iter.next().unwrap())
                        .ok()
                        .expect(HTML_ERROR);
                    for class in classes_iter {
                        if !class.is_empty() {
                            write!(&mut html, " {}", &*class).ok().expect(HTML_ERROR);
                        }
                    }
                    write!(&mut html, "\"").ok().expect(HTML_ERROR);
                }

                if !element.attributes().is_empty() {
                    for (key, value) in element.attributes() {
                        if !key.is_empty() {
                            if !value.is_empty() {
                                write!(&mut html, " {key}=\"{value}\"", key = key, value = value)
                                    .ok()
                                    .expect(HTML_ERROR);
                            } else {
                                write!(&mut html, " {}", key).ok().expect(HTML_ERROR);
                            }
                        }
                    }
                }

                write!(&mut html, ">").ok().expect(HTML_ERROR);


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
                    write!(&mut html, "{}", self.from_component(resource.clone()))
                        .ok()
                        .expect(HTML_ERROR);

                } else {
                    for child in element.children() {
                        if let Some(rendered_child) = self.render(child) {
                            write!(&mut html, "{}", rendered_child).ok().expect(HTML_ERROR);
                        }
                    }
                }

                write!(&mut html, "</{tag}>", tag = tag).ok().expect(HTML_ERROR);

                Some(String::from_utf8(html).unwrap())
            }
            &Ok(Text(ref text)) => Some(text.clone()),
            &Ok(Variable(ref variable)) => {
                match self.variables.get(variable) {
                    Some(value) => Some(value_to_string(&value)),
                    None => Some(String::new()),
                }
            }
            &Ok(Comp(ref ast)) => unreachable!(),
            &Ok(CompCall(ref component_call)) => Some(self.from_component(component_call.clone())),
            &Ok(Function(ref function)) => unimplemented!(),
            &Err(ref error) => {
                if *error == super::AstError::Eof {
                    return None;
                }
                let (index, token_length) = error.values();
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
                exit!()
            }
        }
    }
}

fn value_to_string(value: &Value) -> String {
    use serde_json::Value::*;
    match value {
        &Null => String::new(),
        &Bool(value) => value.to_string(),
        &I64(value) => value.to_string(),
        &U64(value) => value.to_string(),
        &F64(value) => value.to_string(),
        &String(ref string) => string.clone(),
        &Array(ref vector) => {
            let mut concated_string = String::new();
            for value in vector {
                concated_string.push_str(&*value_to_string(value));
            }
            concated_string
        }
        &Object(ref object) => {
            let mut concated_string = String::new();
            for (_, ref value) in object {
                concated_string.push_str(&*value_to_string(*value));
            }
            concated_string
        }
    }
}
#[allow(dead_code)]
mod tests {
    use super::Codegen;
    use std::fs::File;
    use std::io::Read;
    use std::collections::BTreeMap;
    use serde_json::Value;

    const BASIC: &'static str = "<!DOCTYPE html><html><body><p>Hello World!</p></body></html>";
    fn read_file(file_name: &str, json: Value) -> String {
        let mut file = File::open(file_name)
                           .ok()
                           .expect("File doesn't exist, or isn't a file.");
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).ok().expect("File contents corrupted");
        let html = Codegen::codegen(&*file_contents, file_name, json);
        println!("{:#?}", html);
        html
    }

    #[test]
    fn element() {
        assert_eq!(read_file("./tests/element.poly", Value::Object(BTreeMap::new())),
                   BASIC);
    }

    #[test]
    fn component() {

        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        json.insert("world".to_owned(), Value::String("World".to_owned()));
        assert!(!read_file("./tests/component.poly", Value::Object(json)).is_empty());
    }
}
