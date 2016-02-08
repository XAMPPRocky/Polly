use std::cell::RefCell;
use std::collections::BTreeMap;
use std::process;

use serde_json::Value;
use super::*;
use template::Template;

macro_rules! exit {
    () => {{
      if cfg!(debug_assertions) {
          panic!()
      } else {
          process::exit(101);
      }  
    }}
}

pub struct Codegen<'a> {
    parent: RefCell<Template<'a>>,
    elements: Vec<AstResult>,
}

impl<'a> Codegen<'a> {
    fn new(template: &'a Template, ast: Vec<AstResult>, json: Value) -> Self {
        if let Value::Object(object) = json {
            Codegen {
                parent: template,
                elements: ast,
            }
        } else {
            println!("JSON wasn't valid. JSON: {:?}", json);
            exit!();
        }
    }

    pub fn codegen(template: &Template, source: &str, file: &str, json: Value) -> String {
        let lexer = Lexer::lex(source);
        let parser = Parser::from_lexer(template, &lexer);
        let mut codegen = Codegen::new(template, parser.output(), json);
        codegen.to_html()
    }

    pub fn call_component(template: &Template, component: &Component, arg_map: Option<BTreeMap<String, Value>>) -> String {
        let mut codegen = if let Some(arg_map) = arg_map {
            Codegen::new(template, component.ast(), Value::Object(arg_map))
        } else {
            Codegen::new(template, component.ast(), Value::Object(BTreeMap::new()))
        };

        codegen.to_html()
    }

    fn to_html(&mut self) -> String {
        let mut html = String::new();

        for element in self.elements.iter() {
            if let Some(string) = self.render(element) {
                html.push_str(&*string);
            } else {
                break;
            }
        }
        html
    }

    fn get_component(&self, name: String) -> Component {
        self.parent.borrow_mut().get_component(&*name)
    }

    fn from_component(&self, component_call: ComponentCall) -> String {
        if let Some(component) = self.get_component(component_call.name()) {
            let args = component.args();
            let arg_values = component_call.values();
            let mut arg_map = BTreeMap::new();
            if args.len() == arg_values.len() {
                let zipped = args.iter().zip(arg_values.iter());

                for (ref arg, ref value) in zipped {

                    match *arg {
                        &ArgKey::Json(ref arg_name) => {
                            if let &&ArgKey::Json(ref arg_value) = value {
                                let value = match self.parent.variables().get(&*arg_value) {
                                    Some(text) => text.clone(),
                                    None => Value::Null,
                                };
                                arg_map.insert(arg_name.clone(),
                                               Value::String(value_to_string(&value)));
                            }
                        }
                        &ArgKey::Comp(ref arg_name) => {
                            println!("Components can't be passed to other components, they are \
                                      global so you shouldn't need to do it.");
                            exit!();
                        }
                    }
                }
                let mut codegen = Codegen {
                    elements: component.ast(),
                    ..*self.clone()
                };
                codegen.to_html()
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
                write!(&mut html, "<{}", tag).ok().expect(HTML_ERROR);

                if !element.classes().is_empty() {

                    write!(&mut html, " class=\"").ok().expect(HTML_ERROR);
                    let mut classes_iter = element.classes().iter();
                    write!(&mut html, "{}", classes_iter.next().unwrap()).ok().expect(HTML_ERROR);

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
                                write!(&mut html, " {}=\"{}\"", key, value).ok().expect(HTML_ERROR);
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

                write!(&mut html, "</{}>", tag).ok().expect(HTML_ERROR);

                Some(String::from_utf8(html).unwrap())
            }
            &Ok(Text(ref text)) => Some(text.clone()),
            &Ok(Variable(ref variable)) => {
                match self.parent.variables().get(variable) {
                    Some(value) => Some(value_to_string(&value)),
                    None => Some(String::new()),
                }
            }
            &Ok(Comp(ref ast)) => unreachable!(),
            &Ok(CompCall(ref component_call)) => Some(self.from_component(component_call.clone())),
            &Ok(Function(ref function)) => {
                println!("{:#?}", function);
                let components = self.parent.components();
                let mut arguments: BTreeMap<String, ArgValue> = BTreeMap::new();

                for (key, value) in function.args().clone() {

                    match value {
                        ArgKey::Json(id) => {
                            let real_value = self.parent.variables().get(&*id);
                            let real_value = match real_value {
                                Some(value) => Some(value.clone()),
                                None => None,
                            };
                            arguments.insert(key, ArgValue::Json(real_value));
                        }
                        ArgKey::Comp(id) => {
                            let real_value = components.get(&*id);
                            let real_value = match real_value {
                                Some(value) => Some(value.clone()),
                                None => None,
                            };
                            arguments.insert(key, ArgValue::Comp(real_value));
                        }
                    }
                }

                if let Some(fun) = self.parent.get_function(function.identifier()) {
                    match fun(arguments) {
                        Ok(string) => Some(string),
                        Err(error) => {
                            println!("{}", error);
                            exit!();
                        }
                    }
                } else {
                    println!("Function doesn't exist.");
                    exit!();
                }
            }
            &Err(ref error) => {
                if *error == super::AstError::Eof {
                    return None;
                }
                let (index, token_length) = error.values();
                let mut line_number: usize = 0;
                let mut col_number: usize = 1;

                for ch in self.parent.source()[..index].chars() {
                    col_number += 1;
                    if ch == '\n' {
                        line_number += 1;
                        col_number = 1;
                    }
                }

                let mut section = String::new();
                for ch in self.parent.source()[..index].chars().rev() {
                    if ch == '\n' {
                        section = section.chars().rev().collect();
                        break;
                    } else {
                        section.push(ch);
                    }
                }

                for ch in self.parent.source()[index..].chars() {
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
                let file_name_print = format!("{}:{}:{}:", self.parent.file(), line_number, col_number);
                println!("{} {}", file_name_print, error);
                println!("{} {}", file_name_print, section.trim());
                println!("{0:>1$}",
                         underline,
                         2 + col_number + file_name_print.len() -
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

