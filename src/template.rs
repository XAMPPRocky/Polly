use std::collections::{HashMap, BTreeMap};
use std::io::prelude::*;
use std::fs::File;
use std::sync::{Arc, Mutex, MutexGuard};
use std::path::{Path, PathBuf};

use serde_json::Value;

use compiler::tokens::{Component, ArgValue};
use compiler::codegen::Codegen;
use compiler::parser::Parser;
use compiler::lexer::Lexer;
pub type PolyFn = Box<Fn(BTreeMap<String, ArgValue>) -> Result<String, String> + Send>;

macro_rules! std_functions {
    () => {{
        let map: HashMap<&str, PolyFn> = HashMap::new();
        let each = |args: BTreeMap<String, ArgValue>| {
            let mut output = String::new();
            if let Some(&ArgValue::Json(Some(Value::Array(ref array)))) = args.get("array") {
                if let Some(&ArgValue::Comp(Some(ref component))) = args.get("component") {
                    match component.number_of_args() {
                        0 => {
                            for _ in array {
                                output.push_str(&*Template::call_component(&component));
                            }
                            Ok(output)
                        },
                          1 => {
                              let name = component.args()[0].value();
                              for item in array {
                                  let mut map = BTreeMap::new();
                                  map.insert(name.clone(), item.clone());
                                  output.push_str(&*Template::call_component_with_args(&component, map));
                              }
                              Ok(output)   
                          }
                        _ => {
                            if let Some(&Value::Object(_)) = array.first() {
                                let mut iter = array.iter();
                                while let Some(&Value::Object(ref object)) = iter.next() {
                                    let mut map = BTreeMap::new();
                                    for key in component.args() {
                                        let key = key.value();
                                        if let Some(value) = object.get(&*key) {
                                            map.insert(key, value.clone());
                                        } 
                                    }
                                    output.push_str(&*Template::call_component_with_args(&component, map));
                                }
                                Ok(output)
                            } else {
                                Err(String::from("JSON wasn't an object, and the component has\
                                            multiple arguments, so it can't be properly destructured."))
                            }
                        } 
                    }
                } else {
                    Err(String::from("type passed in wasn't a component"))
                }

            } else {
                Err(format!("type passed in wasn't an array it was: {:#?}", args.get("array")))
            }
        };
        map.insert("each", Box::new(each));
        map
    }}
}

pub struct Template<'a>{
    components: HashMap<&'a str, Component>,
    file: PathBuf,
    functions: HashMap<&'a str, PolyFn>,
    source: &'a str,
    variables: BTreeMap<String, Value>,
}


impl<'a> Template<'a> {
    pub fn load<P: AsRef<Path>>(file_path: P) -> Self {
        let file = File::open(file_path).unwrap();
        let source = {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            contents
        };

        Template {
            components: HashMap::new(),
            file: file_path.as_ref().to_path_buf(),
            functions: std_functions!(),
            source: &*source,
            variables: BTreeMap::new(),
        }
    }

    pub fn json(self, json: BTreeMap<String, Value>) -> Self {
        self.variables = json;
        self
    }

    pub fn render(&'a mut self) -> String {
        let lexer = Lexer::new(self.source);
        let parser = Parser::new(lexer.output());
        {self.add_components(parser.get_components());}
        let file_name = self.file.file_name().unwrap().to_str().unwrap();
        let mut codegen = Codegen::new(parser.output(), file_name, self.source, self.variables,self.components, self.functions);
        codegen.to_html()
    }

    pub fn register_function(&'a mut self, name: &'a str, function: PolyFn) {
        if let Some(_) = self.functions.insert(name, function) {
            panic!("Function already exists!");
        }
    }

    pub fn add_components(&'a mut self, mut components: HashMap<&'a str, Component>) {
        for (key, value) in components.drain() {
            if let Some(_) = self.components.insert(key, value) {
                panic!("Component was already defined.");
            }
        }
    }

    pub fn call_component(component: &Component) -> String {
        Codegen::call_component(component, None)
    }

    pub fn call_component_with_args(component: &Component, map: BTreeMap<String, Value>) -> String {
        Codegen::call_component(component, Some(map))
    }
}

#[allow(dead_code, unused_imports)]
mod tests {
    use super::Template;
    use std::fs::File;
    use std::io::Read;
    use std::collections::BTreeMap;
    use serde_json::Value;
    use serde_json::Value::*;

    const BASIC: &'static str = "<!DOCTYPE html><html><body><p>Hello World!</p></body></html>";

    #[test]
    fn element() {
        assert_eq!(Template::load("./tests/element.poly").render(), BASIC);
    }

    #[test]
    fn component() {
        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        json.insert("world".to_owned(), String("World".to_owned()));
        assert_eq!(Template::load("./tests/component.polly").json(json).render(), BASIC);
    }

    #[test]
    fn function() {
        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        let expected = "<!DOCTYPE \
                        html><html><body><ul><li>Rust</li><li>C++</li><li>JavaScript</li></ul></bo\
                        dy></html>";
        json.insert("array".to_owned(),
                    Array(vec![String("Rust".to_owned()),
                               String("C++".to_owned()),
                               String("JavaScript".to_owned())]));
        assert_eq!(Template::load("./tests/function.polly").json(json).render(), expected);

    }

}
