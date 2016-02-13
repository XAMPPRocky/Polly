use std::cell::RefCell;
use std::collections::{HashMap, BTreeMap};
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::Value;

use compiler::{ArgValue, Codegen, Component, Lexer, Parser};

/// A type abstracting the functions used for Polly.
pub type PolyFn = Box<Fn(BTreeMap<String, ArgValue>, &Rc<RefCell<Template>>)
                         -> Result<String, String>>;

fn std_functions() -> HashMap<String, PolyFn> {
    use serde_json::Value;
    use compiler::tokens::ArgValue::*;

    let mut map: HashMap<String, PolyFn> = HashMap::new();

    map.insert(String::from("std.each"), Box::new(|args: BTreeMap<String, ArgValue>, parent| {
        let mut output = String::new();
        if let Some(&Json(Some(Value::Array(ref array)))) = args.get("array") {
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
                        if let Some(&Value::Object(_)) = array.first() {
                            let iter = array.iter();
                            for json in iter {
                                if let Value::Object(ref object) = *json {
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
                       let condition = match *json {
                           Value::Array(ref array) => !array.is_empty(),
                           Value::Null => false,
                           Value::Bool(ref boolean) => *boolean,
                           Value::I64(ref num) => *num != 0,
                           Value::U64(ref num) => *num != 0,
                           Value::F64(ref num) => *num != 0.0,
                           Value::String(ref string) => !string.is_empty(),
                           Value::Object(ref object) => !object.is_empty(),
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
                                               &Value::Object(ref object) => {
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
                                           if let Value::Object(ref map) = *json {
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

/// The Polly template.
pub struct Template {
    components: HashMap<String, Component>,
    file: PathBuf,
    functions: HashMap<String, PolyFn>,
    source: String,
    locales_dir: Option<String>,
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

    /// Get a component from within the template.
    pub fn get_component(&self, name: &str) -> Option<&Component> {
        self.components.get(name)
    }

    /// Get a function from within the template.
    pub fn get_function(&self, name: &str) -> Option<&PolyFn> {
        self.functions.get(name)
    }

    /// Loads the template from the file path.
    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self, TemplateError> {
        let source = match Template::read_to_source(file_path.as_ref()) {
            Ok(source) => source,
            Err(error) => return Err(error),
        };

        Ok(Template::new(file_path, source))
    }

    fn new<P: AsRef<Path>, S: Into<String>>(path: P, source: S) -> Self {
        Template {
            components: HashMap::new(),
            file: path.as_ref().to_path_buf(),
            functions: std_functions(),
            source: source.into(),
            locales_dir: Some(String::from("./templates/locales")),
            variables: BTreeMap::new(),
        }
    }

    /// Specify that a template has no locales available.
    pub fn no_locales(mut self) -> Self {
        self.locales_dir = None;
        self
    }

    /// Loads the template from the source provided. The file path is also required, for error 
    /// handling
    pub fn load_from_source<P: AsRef<Path>, S: Into<String>>(path: P, source: S) -> Self {
        Template::new(path, source)
    }

    /// Pass in a `serde_json` Object, for the JSON of the template.
    pub fn json(mut self, json: BTreeMap<String, Value>) -> Self {
        self.variables = json;
        self
    }

    /// Override the default locales directory.
    pub fn locales_dir<S: Into<String>>(mut self, locales_dir: S) -> Self {
        self.locales_dir = Some(locales_dir.into());
        self
    }

    /// Registers a function to the template.
    pub fn register(&mut self, name: String, function: PolyFn) -> Result<(), TemplateError> {
        if let Some(_) = self.functions.insert(name, function) {
            Err(TemplateError::PreDefinedFunction)
        } else {
            Ok(())
        }
    }

    pub fn import<P: AsRef<Path>>(&mut self, path: P) -> Result<(), TemplateError> {
        match Template::read_to_source(path) {
            Ok(source) => {
                for (key, value) in Parser::component_pass(Lexer::new(&*source).output()) {
                    self.add_component(key, value);
                }
                Ok(())
            }
            Err(error) => return Err(error),
        }
    }

    fn read_to_source<P: AsRef<Path>>(path: P) -> Result<String, TemplateError> {
        println!("{:?}", path.as_ref());
        let mut file = File::open(path.as_ref()).unwrap();
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => Ok(contents),
            Err(error) => Err(TemplateError::IoError(error)),
        }
    }

    /// Renders the template into a HTML String.
    pub fn render(mut self, lang: &str) -> Result<String, TemplateError> {
        let output = {
            let parser = Parser::new(Lexer::new(&self.source).output());
            self.add_components(parser.get_components());
            parser.output()
        };
        let file_name = self.file.file_name().unwrap().to_str().unwrap().to_owned();

        let locales_dir = match self.locales_dir {
            Some(ref locales_dir) => locales_dir.clone(),
            None => String::new(),
        };

        if !locales_dir.is_empty() {
            let path = format!("{dir}/{lang}/{file}",
                               dir = locales_dir,
                               lang = lang,
                               file = file_name);
            match Template::read_to_source(path) {
                Ok(source) => {
                    for (key, value) in Parser::component_pass(Lexer::new(&*source).output()) {
                        let new_key = format!("locales.{}", key);
                        self.add_component(new_key, value);
                    }
                }
                error => return error,
            }
        }

        let source = self.source.to_owned();
        let variables = self.variables.to_owned();

        let mut codegen = Codegen::new(output,
                                       file_name,
                                       source,
                                       variables,
                                       Rc::new(RefCell::new(self)));
        Ok(codegen.generate_html())
    }
}
/// Errors relating to the templating rendering.
#[derive(Debug)]
pub enum TemplateError {
    /// The function called already exists.
    PreDefinedFunction,
    /// Any IO errors, from the methods.
    IoError(io::Error),
}

#[allow(dead_code, unused_imports)]
mod tests {
    use super::Template;
    use std::fs::File;
    use std::io::Read;
    use std::collections::BTreeMap;
    use serde_json::Value;

    const BASIC: &'static str = "<!DOCTYPE html><html><body><p>Hello World!</p></body></html>";
    const BASIC_DE: &'static str = "<!DOCTYPE html><html><body><p>Hallo Welt!</p></body></html>";
    #[test]
    fn element() {
        assert_eq!(Template::load("./tests/element.polly")
                       .unwrap()
                       .no_locales()
                       .render("en")
                       .unwrap(),
                   BASIC);
    }

    #[test]
    fn component() {
        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        json.insert("world".to_owned(), Value::String("World".to_owned()));
        let result = Template::load("./tests/component.polly")
                         .unwrap()
                         .no_locales()
                         .json(json)
                         .render("en");
        assert_eq!(result.unwrap(), BASIC);
    }

    #[test]
    fn component_imported() {
        let mut template = Template::load("./tests/component_import.polly").unwrap().no_locales();
        template.import("./tests/imported.polly").unwrap();
        assert_eq!(template.render("en").unwrap(), BASIC);
    }


    #[test]
    fn variable() {
        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        json.insert("world".to_owned(), Value::String("World".to_owned()));
        let result = Template::load("./tests/variable.polly")
                         .unwrap()
                         .no_locales()
                         .json(json)
                         .render("en");
        assert_eq!(result.unwrap(), BASIC);
    }

    #[test]
    fn function_each() {
        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        let expected = "<!DOCTYPE \
                        html><html><body><ul><li>Rust</li><li>C++</li><li>JavaScript</li></ul></bo\
                        dy></html>";
        json.insert("array".to_owned(),
                    Value::Array(vec![Value::String("Rust".to_owned()),
                                      Value::String("C++".to_owned()),
                                      Value::String("JavaScript".to_owned())]));
        assert_eq!(Template::load("./tests/function_each.polly")
                       .unwrap()
                       .no_locales()
                       .json(json)
                       .render("en")
                       .unwrap(),
                   expected);

    }

    #[test]
    fn function_if() {
        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        json.insert(String::from("condition"), Value::Bool(true));
        json.insert(String::from("text"),
                    Value::String(String::from("Hello World!")));
        assert_eq!(Template::load("./tests/function_if.polly")
                       .unwrap()
                       .no_locales()
                       .json(json)
                       .render("en")
                       .unwrap(),
                   BASIC);

    }

    #[test]
    fn locales() {
        let json: BTreeMap<String, Value> = BTreeMap::new();

        assert_eq!(Template::load("./tests/locales.polly")
                       .unwrap()
                       .json(json.to_owned())
                       .render("en")
                       .unwrap(),
                   BASIC);
        assert_eq!(Template::load("./tests/locales.polly")
                       .unwrap()
                       .json(json)
                       .render("de")
                       .unwrap(),
                   BASIC_DE);
    }

}
