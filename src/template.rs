use std::cell::RefCell;
use std::collections::{HashMap, BTreeMap};
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::Value;

use compiler::{ArgValue, AstError, Codegen, CodegenError, CodegenResult, Component, Lexer, Parser};

/// A type abstracting the functions used for Polly.
pub type PolyFn = Box<Fn(BTreeMap<String, ArgValue>, &Rc<RefCell<Template>>)
                         -> Result<String, String>>;

macro_rules! template_try {
    ($result:expr) => {
        match $result {
            Ok(html) => return Ok(html),
            Err(error) => return Err(format!("{:#?}", error)),
        }
    }
}

/// # Standard Functions of Polly
/// **Note:** This shouldn't be imported in into your crate, but is exposed as a way to incorporate
/// Polly's functions into the documentation.
///
/// ## std.each
/// **Arguments**
/// 
/// - array - The array of JSON to be iterated over.
/// - component - The component to render for each element in array.
/// 
/// Applies a component to each element in the array. If the component takes more than one argument,
/// the component can only be used with objects(in which case the function will attempt to find the
/// object's property based on the argument name), or arrays of the same length as the number of
/// arguments.
/// 
/// ```
/// extern crate serde_json;
/// extern crate polly;
/// 
/// use serde_json::Value;
/// use polly::Template;
/// fn main() {
///     const EXPECTED: &'static str = "\
///     <html>\
///         <body>\
///             <ul>\
///                 <li>Item 1</li>\
///                 <li>Item 2</li>\
///                 <li>Item 3</li>\
///             </ul>\
///         </body>\
///     </html>";
///     let json: Value = serde_json::from_str(r#"{"array": ["Item 1","Item 2","Item 3"]}"#).unwrap();
///     let json = json.as_object().unwrap().clone();
///     let template = Template::load_from_source("documentation", r#"
///     &listItem(@item) {/li{@item}}
///     /html {
///        /body {
///            /ul{$std.each(component = &listItem, array = @array)}
///        }
///     }
///     "#);
///     
///      assert_eq!(template.json(json).no_locales().render("en").unwrap(), EXPECTED);
/// }
/// ```
///
/// ## std.if
/// **Arguments**
/// 
/// - condition - The condition to be checked.
/// - component - The component to be rendered based on the trueness of the condition.
/// - json(*optional*) - The JSON passed to the component if needed.
/// 
/// Checks the JSON value of condition, and if it is true, renders the component, optinally passing
/// in JSON for the component. It is important to note that what counts as true is similar to \
/// JavaScript, so it doesn't have to be strictly a boolean.
/// 
/// **Boolean conditions**
/// 
/// - Array - false if the array's length is 0.
/// - Null - Always false.
/// - Bool - The literal value.
/// - I64 - false if = 0.
/// - U64 - false if = 0.
/// - F64 - false if = 0.0.
/// - String - false if the string is empty.
/// - Object - false if the object is empty.
/// 
/// ```
/// extern crate serde_json;
/// extern crate polly;
/// 
/// use serde_json::Value;
/// use polly::Template;
/// fn main() {
///     const EXPECTED: &'static str = "\
///     <html>\
///         <body>\
///         </body>\
///     </html>";
///     let json: Value = serde_json::from_str(r#"{"bool": false}"#).unwrap();
///     let json = json.as_object().unwrap().clone();
///     let template = Template::load_from_source("documentation", r#"
///     &component {/h1{Hello World!}}
///     /html {
///        /body {
///            $std.if(component = &component, condition = @bool)
///        }
///     }
///     "#);
///     
///      assert_eq!(template.json(json).no_locales().render("en").unwrap(), EXPECTED);
/// }
/// ```
/// 
/// ## std.if_else
/// **Arguments**
/// 
/// - condition - The condition to be checked.
/// - component - The component to be rendered based on the trueness of the condition.
/// - else - The component rendered if the condition is false.
/// - json(*optional*) - The JSON passed to the component, or the else component if needed.
/// 
/// Follows the same rules as **std.if** except with the ability to render a component if the value
/// is false. 
/// 
/// ```
/// extern crate serde_json;
/// extern crate polly;
/// 
/// use serde_json::Value;
/// use polly::Template;
/// fn main() {
///     const EXPECTED: &'static str = "\
///     <html>\
///         <body>\
///             <h1>Goodbye World!</h1>\
///         </body>\
///     </html>";
///     let json: Value = serde_json::from_str(r#"{"bool": false}"#).unwrap();
///     let json = json.as_object().unwrap().clone();
///     let template = Template::load_from_source("documentation", r#"
///     &hello {Hello}
///     &goodbye {Goodbye}
///     /html {
///        /body {
///            /h1 {$std.if_else(component = &hello, condition = @bool, else = &goodbye) World!}
///        }
///     }
///     "#);
///     
///      assert_eq!(template.json(json).no_locales().render("en").unwrap(), EXPECTED);
/// }
/// ```
pub fn std_functions() -> HashMap<String, PolyFn> {
    use serde_json::Value;
    use compiler::tokens::ArgValue::*;

    let mut map: HashMap<String, PolyFn> = HashMap::new();

    map.insert(String::from("std.each"), Box::new(|args, parent| {
        let mut output = String::new();
        if let Some(&Json(Some(Value::Array(ref array)))) = args.get("array") {
            if let Some(&Comp(Some(ref component))) = args.get("component") {
                match component.number_of_args() {
                    0 => {
                        for _ in array {
                            match Template::call_component(&component, parent) {
                                Ok(html) => output.push_str(&*html),
                                Err(error) => return Err(format!("{:#?}", error)),
                            }
                        }
                        Ok(output)
                    }
                    1 => {
                        let name = component.args().first().unwrap().value();
                        for item in array {
                            let mut map = BTreeMap::new();
                            map.insert(name.clone(), item.clone());
                            match Template::call_component_with_args(&component, parent, map) {
                                Ok(html) => output.push_str(&*html),
                                Err(error) => return Err(format!("{:#?}", error)),
                            }
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
                                    match Template::call_component_with_args(&component, parent, map) {
                                        Ok(html) => output.push_str(&*html),
                                        Err(error) => return Err(format!("{:#?}", error)),
                                    }
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
                       if json_into_bool(json) {
                           eval_conditional(&args, &parent, "component")
                       } else {
                           Ok(String::new())
                       }
                   } else {
                       Err(format!("The json arg, wasn't JSON it is {:#?}",
                                   args.get("condition")))
                   }
               }));

    map.insert(String::from("std.if_else"),
               Box::new(|args, parent| {
                   if let Some(&Json(Some(ref json))) = args.get("condition") {
                       if json_into_bool(json) {
                           eval_conditional(&args, &parent, "component")
                       } else {
                           eval_conditional(&args, &parent, "else")
                       }
                   } else {
                       Err(format!("The json arg, wasn't JSON it is {:#?}",
                                   args.get("condition")))
                   }

               }));

    fn eval_conditional(args: &BTreeMap<String, ArgValue>,
                        parent: &Rc<RefCell<Template>>,
                        component_name: &str)
                        -> Result<String, String> {
        if let Some(&Comp(Some(ref component))) = args.get(component_name) {
            if let Some(&Json(Some(ref json))) = args.get("json") {
                match component.number_of_args() {
                    0 => template_try!(Template::call_component(component, parent)),
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
                        template_try!(Template::call_component_with_args(component, parent, map));
                    }
                    _ => {
                        if let Value::Object(ref map) = *json {
                            template_try!(Template::call_component_with_args(component,
                                                                             parent,
                                                                             map.clone()));
                        } else {
                            Err(String::from("Component has more than one argument, and the JSON \
                                              passed in wasn't an object, so I don't know how to \
                                              destructure it."))
                        }
                    }
                }
            } else {
                template_try!(Template::call_component(component, parent))
            }
        } else {
            Err(format!("The component arg, wasn't a component it was a {:#?}",
                        args.get("component")))
        }
    }
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
    fn add_components(&mut self,
                      mut components: HashMap<String, Component>)
                      -> Result<(), TemplateError> {
        for (key, value) in components.drain() {
            if let Some(_) = self.components.insert(key, value) {
                return Err(TemplateError::PreDefinedComponent);
            }
        }
        Ok(())
    }

    fn add_component(&mut self, key: String, value: Component) -> Result<(), TemplateError> {
        if let Some(_) = self.components.insert(key, value) {
            return Err(TemplateError::PreDefinedComponent);
        } else {
            return Ok(());
        }
    }

    fn call_component(component: &Component, parent: &Rc<RefCell<Template>>) -> CodegenResult {
        Codegen::call_component(component, None, parent)
    }

    fn call_component_with_args(component: &Component,
                                parent: &Rc<RefCell<Template>>,
                                map: BTreeMap<String, Value>)
                                -> CodegenResult {
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

    /// Imports components from another template.
    pub fn import<P: AsRef<Path>>(&mut self, path: P) -> Result<(), TemplateError> {
        match Template::read_to_source(path) {
            Ok(source) => {
                for (key, value) in Parser::component_pass(Lexer::new(&*source).output()) {
                    if let Err(error) = self.add_component(key, value) {
                        return Err(error);
                    };
                }
                Ok(())
            }
            Err(error) => return Err(error),
        }
    }

    fn read_to_source<P: AsRef<Path>>(path: P) -> Result<String, TemplateError> {
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
            if let Err(error) = self.add_components(parser.get_components()) {
                return Err(error);
            };
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
                        if let Err(error) = self.add_component(new_key, value) {
                            return Err(error);
                        };
                    }
                }
                error => return error,
            }
        }

        let variables = self.variables.to_owned();

        let mut codegen = Codegen::new(output, variables, Rc::new(RefCell::new(self)));
        match codegen.generate_html() {
            Ok(html) => Ok(html),
            Err(error) => Err(TemplateError::CodegenError(error)),
        }
    }
    /// Renders the component, or panics if there is an error.
    pub fn unwrap_render(self, locale: &str) -> String {
        let file_name = self.file.file_name().unwrap().to_str().unwrap().to_owned();
        let source = self.source.clone();
        match self.render(locale) {
            Ok(html) => html,
            Err(error) => {
                Template::render_error(error, source, file_name);
                panic!()
            }
        }
    }

    fn render_error(error: TemplateError, source: String, file_name: String) {
        if let TemplateError::CodegenError(CodegenError::AstError(error)) = error {
            const SPACE: usize = 1;
            const CONSOLE_WIDTH: usize = 89;
            if error == AstError::Eof {
                return;
            }
            let (index, token_length) = error.values();
            let mut line_number: usize = 0;
            let mut col_number: usize = 1;

            for ch in source[..index].chars() {
                col_number += 1;
                if ch == '\n' {
                    line_number += 1;
                    col_number = 1;
                }
            }

            let mut section = String::new();
            for ch in source[..index].chars().rev() {
                if ch == '\n' {
                    section = section.chars().rev().collect();
                    break;
                } else {
                    section.push(ch);
                }
            }

            let file_name_print = format!("{}:{}:{}:", file_name, line_number, col_number);

            for ch in source[index..].chars() {
                if ch == '\n' || (section.len() + SPACE + file_name_print.len()) == CONSOLE_WIDTH {
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
            println!("{} {}", file_name_print, error);
            println!("{} {}", file_name_print, section.trim());
            println!("{0:>1$}",
                     underline,
                     SPACE + col_number + file_name_print.len() -
                     (section.len() - section.trim().len()));
        } else {
            println!("{:?}", error);
        }
    }
}
/// Errors relating to the templating rendering.
#[derive(Debug)]
pub enum TemplateError {
    /// Error within the Code generation.
    CodegenError(CodegenError),
    /// The component called already exists.
    PreDefinedComponent,
    /// The function called already exists.
    PreDefinedFunction,
    /// Any IO errors, from the methods.
    IoError(io::Error),
}
fn json_into_bool(json: &Value) -> bool {
    match *json {
        Value::Array(ref array) => !array.is_empty(),
        Value::Null => false,
        Value::Bool(ref boolean) => *boolean,
        Value::I64(ref num) => *num != 0,
        Value::U64(ref num) => *num != 0,
        Value::F64(ref num) => *num != 0.0,
        Value::String(ref string) => !string.is_empty(),
        Value::Object(ref object) => !object.is_empty(),
    }
}
#[allow(dead_code, unused_imports)]
mod tests {
    use super::Template;
    use std::fs::File;
    use std::io::Read;
    use std::collections::BTreeMap;
    use serde_json;
    use serde_json::Value;

    const BASIC: &'static str = "<!DOCTYPE html><html><body><p>Hello World!</p></body></html>";
    const BASIC_DE: &'static str = "<!DOCTYPE html><html><body><p>Hallo Welt!</p></body></html>";
    #[test]
    fn element() {
        assert_eq!(Template::load("./tests/element.polly")
                       .unwrap()
                       .no_locales()
                       .unwrap_render("en"),
                   BASIC);
    }

    #[test]
    fn component() {
        let json: Value = serde_json::from_str(r#"{"world": "World"}"#).unwrap();

        let template = Template::load("./tests/component.polly")
                           .unwrap()
                           .no_locales()
                           .json(json.as_object().unwrap().to_owned())
                           .render("en");
        assert_eq!(template.unwrap(), BASIC);
    }

    #[test]
    fn component_imported() {
        let mut template = Template::load("./tests/component_import.polly").unwrap().no_locales();
        template.import("./tests/imported.polly").unwrap();
        assert_eq!(template.unwrap_render("en"), BASIC);
    }


    #[test]
    fn variable() {
        let json: Value = serde_json::from_str(r#"{"world": "World"}"#).unwrap();

        let template = Template::load("./tests/variable.polly")
                           .unwrap()
                           .no_locales()
                           .json(json.as_object().unwrap().to_owned());
        assert_eq!(template.unwrap_render("en"), BASIC);
    }

    #[test]
    fn variable_inside_another_variable() {
        let json: Value = serde_json::from_str(r#"{"object": {"world": "World"}}"#).unwrap();

        let template = Template::load("./tests/variable_inside.polly")
                           .unwrap()
                           .no_locales()
                           .json(json.as_object().unwrap().to_owned());
        assert_eq!(template.unwrap_render("en"), BASIC);
    }

    #[test]
    fn function_each() {
        let expected = "<!DOCTYPE \
                        html><html><body><ul><li>Rust</li><li>C++</li><li>JavaScript</li></ul></bo\
                        dy></html>";
        let json: Value = serde_json::from_str(r#"{"array": ["Rust", "C++", "JavaScript"]}"#)
                              .unwrap();

        assert_eq!(Template::load("./tests/function_each.polly")
                       .unwrap()
                       .no_locales()
                       .json(json.as_object().unwrap().to_owned())
                       .unwrap_render("en"),
                   expected);

    }

    #[test]
    fn function_if() {
        let json: Value = serde_json::from_str(r#"{"condition": true, "text": "Hello World!"}"#)
                              .unwrap();

        assert_eq!(Template::load("./tests/function_if.polly")
                       .unwrap()
                       .no_locales()
                       .json(json.as_object().unwrap().to_owned())
                       .unwrap_render("en"),
                   BASIC);

    }

    #[test]
    fn function_if_else() {
        let json: Value = serde_json::from_str(r#"{"condition": false, "text": "Hello World!"}"#)
                              .unwrap();

        assert_eq!(Template::load("./tests/function_if_else.polly")
                       .unwrap()
                       .no_locales()
                       .json(json.as_object().unwrap().to_owned())
                       .unwrap_render("en"),
                   BASIC);

    }


    #[test]
    fn locales() {
        let json: BTreeMap<String, Value> = BTreeMap::new();

        assert_eq!(Template::load("./tests/locales.polly")
                       .unwrap()
                       .locales_dir("./tests/locales/")
                       .json(json.to_owned())
                       .unwrap_render("en"),
                   BASIC);
        assert_eq!(Template::load("./tests/locales.polly")
                       .unwrap()
                       .locales_dir("./tests/locales/")
                       .json(json)
                       .unwrap_render("de"),
                   BASIC_DE);
    }

}
