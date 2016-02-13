use std::cell::RefCell;
use std::collections::{HashMap, BTreeMap};
use std::io::prelude::*;
use std::fs::File;
use std::rc::Rc;
use std::path::{Path, PathBuf};

use serde_json::Value;

use compiler::tokens::{Component, ArgValue};
use compiler::codegen::Codegen;
use compiler::parser::Parser;
use compiler::lexer::Lexer;

/// A type abstracting the functions used for Polly.
pub type PolyFn = Box<Fn(BTreeMap<String, ArgValue>, &Rc<RefCell<Template>>)
                         -> Result<String, String>>;

fn std_functions() -> HashMap<String, PolyFn> {
    use serde_json::Value::*;
    use compiler::tokens::ArgValue::*;

    let mut map: HashMap<String, PolyFn> = HashMap::new();

    map.insert(String::from("std.each"), Box::new(|args: BTreeMap<String, ArgValue>, parent| {
        let mut output = String::new();
        if let Some(&Json(Some(Array(ref array)))) = args.get("array") {
            if let Some(&Comp(Some(ref component))) = args.get("component") {
                match component.number_of_args() {
                    0 => {
                        for _ in array {
                            output.push_str(&*Template::call_component(&component, parent));
                        }
                        Ok(output)
                    }
                    1 => {
                        let name = component.args().first().unwrap().value();
                        for item in array {
                            let mut map = BTreeMap::new();
                            map.insert(name.clone(), item.clone());
                            output.push_str(&*Template::call_component_with_args(component,
                                                                                 parent,
                                                                                 map));
                        }
                        Ok(output)
                    }
                    _ => {
                        if let Some(&Object(_)) = array.first() {
                            let mut iter = array.iter();
                            while let Some(&Object(ref object)) = iter.next() {
                                let mut map = BTreeMap::new();
                                for key in component.args() {
                                    let key = key.value();
                                    if let Some(value) = object.get(&*key) {
                                        map.insert(key, value.clone());
                                    }
                                }
                                output.push_str(&*Template::call_component_with_args(&component,
                                                                                     parent,
                                                                                     map));
                            }
                            Ok(output)
                        } else {
                            Err(String::from("JSON wasn't an object, and the component has \
                                              multiple arguments, so it can't be properly \
                                              destructured."))
                        }
                    } 
                }
            } else {
                Err(String::from("type passed in wasn't a component"))
            }

        } else {
            Err(format!("type passed in wasn't an array it was: {:#?}",
                        args.get("array")))
        }
    }));

    map.insert(String::from("std.if"),
               Box::new(|args, parent| {

                   if let Some(&Json(Some(ref json))) = args.get("condition") {
                       let condition = match json {
                           &Array(ref array) => !array.is_empty(),
                           &Null => false,
                           &Bool(ref boolean) => boolean.clone(),
                           &I64(ref num) => *num != 0,
                           &U64(ref num) => *num != 0,
                           &F64(ref num) => *num != 0.0,
                           &String(ref string) => !string.is_empty(),
                           &Object(ref object) => !object.is_empty(),
                       };

                       if condition {
                           if let Some(&Comp(Some(ref component))) = args.get("component") {
                               if let Some(&Json(Some(ref json))) = args.get("json") {
                                   match component.number_of_args() {
                                       0 => Ok(Template::call_component(component, parent)),
                                       1 => {
                                           let name = component.args().first().unwrap().value();
                                           let mut map = BTreeMap::new();
                                           match json {
                                               &Object(ref object) => {
                                                   if let Some(value) = object.get(&name) {
                                                       map.insert(name, value.clone());
                                                   }
                                               }
                                               rest => {
                                                   map.insert(name, rest.clone());
                                               }
                                           }
                                           Ok(Template::call_component_with_args(component,
                                                                                 parent,
                                                                                 map))
                                       }
                                       _ => {
                                           if let &Object(ref map) = json {
                                               Ok(Template::call_component_with_args(component,
                                                                                     parent,
                                                                                     map.clone()))
                                           } else {
                                               Err(String::from("Component has more than one \
                                                                 argument, and the JSON passed \
                                                                 in wasn't an object, so I don't \
                                                                 know how to destructure it."))
                                           }
                                       }
                                   }
                               } else {
                                   Ok(Template::call_component(component, parent))
                               }
                           } else {
                               Err(format!("The component arg, wasn't a component it was a {:#?}",
                                           args.get("component")))
                           }
                       } else {
                           Ok(String::new())
                       }
                   } else {
                       Err(format!("The json arg, wasn't JSON it is {:#?}",
                                   args.get("condition")))
                   }
               }));

    map
}

/// A struct representing a template.
pub struct Template {
    components: HashMap<String, Component>,
    file: PathBuf,
    functions: HashMap<String, PolyFn>,
    source: String,
    locales_dir: String,
    variables: BTreeMap<String, Value>,
}


impl Template {
    fn add_components(&mut self, mut components: HashMap<String, Component>) {
        for (key, value) in components.drain() {
            if let Some(_) = self.components.insert(key, value) {
                panic!("Component was already defined.");
            }
        }
    }

    fn add_component(&mut self, key: String, value: Component) {
        if let Some(_) = self.components.insert(key, value) {
            panic!("Component was already defined.");
        }
    }

    fn call_component(component: &Component, parent: &Rc<RefCell<Template>>) -> String {
        Codegen::call_component(component, None, parent)
    }

    fn call_component_with_args(component: &Component,
                                parent: &Rc<RefCell<Template>>,
                                map: BTreeMap<String, Value>)
                                -> String {
        Codegen::call_component(component, Some(map), parent)
    }

    pub fn get_component(&self, name: &str) -> Option<&Component> {
        self.components.get(name)
    }
    pub fn get_function(&self, name: &str) -> Option<&PolyFn> {
        self.functions.get(name)
    }
    /// loads the template from the file path.
    pub fn load<P: AsRef<Path>>(file_path: P) -> Self {
        let mut file = File::open(file_path.as_ref()).unwrap();
        let source = {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            contents
        };

        Template {
            components: HashMap::new(),
            file: file_path.as_ref().to_path_buf(),
            functions: std_functions(),
            source: source,
            locales_dir: String::from("./templates/locales"),
            variables: BTreeMap::new(),
        }
    }

    /// Pass in a `serde_json` Object, for the JSON of the template.
    pub fn json(mut self, json: BTreeMap<String, Value>) -> Self {
        self.variables = json;
        self
    }

    /// Override the default locales directory.
    pub fn locales_dir<S: Into<String>>(mut self, locales_dir: S) -> Self {
        self.locales_dir = locales_dir.into();
        self
    }

    pub fn register(&mut self, name: String, function: PolyFn) {
        if let Some(_) = self.functions.insert(name, function) {
            panic!("Function already exists!");
        }
    }

    pub fn render(mut self, lang: &str) -> String {
        let output = {
            let output = {
                let lexer = Lexer::lex(&self.source);
                lexer.output()
            };
            let parser = Parser::new(output);
            self.add_components(parser.get_components());
            parser.output()
        };

        let file_name = self.file.file_name().unwrap().to_str().unwrap().to_owned();

        let path = format!("{dir}/{lang}/{file}",
                           dir = self.locales_dir,
                           lang = lang,
                           file = file_name);
        if let Ok(mut file) = File::open(path) {
            let contents = {
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                contents
            };
            let output = {
                let lexer = Lexer::lex(&*contents);
                lexer.output()
            };
            let parser = Parser::new(output);
            for (key, value) in parser.get_components() {
                let new_key = format!("locales.{}", key);
                self.add_component(new_key, value);
            }
        }

        let source = self.source.to_owned();
        let variables = self.variables.to_owned();

        let mut codegen = Codegen::new(output,
                                       file_name,
                                       source,
                                       variables,
                                       Rc::new(RefCell::new(self)));
        codegen.to_html()
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
        assert_eq!(Template::load("./tests/element.polly").render("en"), BASIC);
    }

    #[test]
    fn component() {
        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        json.insert("world".to_owned(), String("World".to_owned()));
        let result = Template::load("./tests/component.polly").json(json).render("en");
        assert_eq!(result, BASIC);
    }


    #[test]
    fn variable() {
        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        json.insert("world".to_owned(), String("World".to_owned()));
        let result = Template::load("./tests/variable.polly").json(json).render("en");
        assert_eq!(result, BASIC);
    }

    #[test]
    fn function_each() {
        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        let expected = "<!DOCTYPE \
                        html><html><body><ul><li>Rust</li><li>C++</li><li>JavaScript</li></ul></bo\
                        dy></html>";
        json.insert("array".to_owned(),
                    Array(vec![String("Rust".to_owned()),
                               String("C++".to_owned()),
                               String("JavaScript".to_owned())]));
        assert_eq!(Template::load("./tests/function_each.polly").json(json).render("en"),
                   expected);

    }

    #[test]
    fn function_if() {
        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        json.insert(String::from("condition"), Bool(true));
        json.insert(String::from("text"), String(String::from("Hello World!")));
        assert_eq!(Template::load("./tests/function_if.polly").json(json).render("en"),
                   BASIC);

    }

    #[test]
    fn locales() {
        let json: BTreeMap<String, Value> = BTreeMap::new();
        const BASIC_DE: &'static str = "<!DOCTYPE html><html><body><p>Hallo \
                                        Welt!</p></body></html>";

        assert_eq!(Template::load("./tests/locales.polly").json(json.to_owned()).render("en"),
                   BASIC);
        assert_eq!(Template::load("./tests/locales.polly").json(json).render("de"),
                   BASIC_DE);

    }

}
