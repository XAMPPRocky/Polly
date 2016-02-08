use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::Value;

use compiler::tokens::{ArgKey, ArgValue, Component};
use compiler::codegen::Codegen;
pub type PolyFn = Box<Fn(BTreeMap<String, ArgValue>) -> Result<String, String>>;

pub struct Template<'a> {
    codegen: Codegen<'a>,
    file_name: &'a str,
    source: &'a str,
    variables: BTreeMap<String, Value>,
    functions: HashMap<&'a str, PolyFn>,
    components: HashMap<String, Component>,
}


impl<'a> Template<'a> {
    pub fn load_with_json(file: &str, json: Value) -> Self {
        unimplemented!()
    }
    pub fn load(file: &str) -> Self {
        unimplemented!()
    }

    pub fn variables(&self) -> &BTreeMap<String, Value> {
        &self.variables
    }

    pub fn file(&self) -> &str {
        self.file_name
    }

    pub fn source(&self) -> &str {
        self.source
    }

    pub fn components(&self) -> &HashMap<String, Component> {
        &self.components
    }

    pub fn insert_component(&mut self, name: String, component: Component){
        self.components.insert(name, component);
    }

    pub fn call_component(&self, component: &Component) -> String {
        Codegen::call_component(self, component, None)
    }

    pub fn call_component_with_args(&self, component: &Component, map: BTreeMap<String, Value>) -> String {
        Codegen::call_component(self, component, Some(map))
    }

    pub fn get_function(&self, name: String) -> Option<&PolyFn> {
        self.functions.get(&*name)
    }
}

#[allow(dead_code)]
mod tests {
    use compiler::Codegen;
    use std::fs::File;
    use std::io::Read;
    use std::collections::BTreeMap;
    use serde_json::Value;

    const BASIC: &'static str = "<!DOCTYPE html><html><body><p>Hello World!</p></body></html>";

    #[test]
    fn element() {
        assert_eq!(Template::load_with_json("./tests/element.poly", Value::Object(BTreeMap::new())),
                   BASIC);
    }

    #[test]
    fn component() {
        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        json.insert("world".to_owned(), Value::String("World".to_owned()));
        assert_eq!(Template::load_with_json("./tests/component.poly", Value::Object(json)),
                   BASIC);
    }

    #[test]
    fn function() {
        use serde_json::Value::*;
        let mut json: BTreeMap<String, Value> = BTreeMap::new();
        let expected = "<!DOCTYPE \
                        html><html><body><ul><li>Rust</li><li>C++</li><li>JavaScript</li></ul></bo\
                        dy></html>";
        json.insert("array".to_owned(),
                    Array(vec![String("Rust".to_owned()),
                               String("C++".to_owned()),
                               String("JavaScript".to_owned())]));
        assert_eq!(Template::load_with_json("./tests/function.polly", Object(json)), expected);

    }

}
