use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, Mutex, MutexGuard};

use serde_json::Value;

use compiler::tokens::{Component, ArgValue};
use compiler::codegen::Codegen;
pub type PolyFn = Fn(BTreeMap<String, ArgValue>) -> Result<String, String> + Send;

lazy_static! {
    static ref COMPONENTS: Mutex<HashMap<String, Component>> = {
        Mutex::new(HashMap::new())
    };
    static ref FUNCTIONS: Arc<Mutex<HashMap<&'static str, Box<PolyFn>>>> = {
        let mut map:HashMap<&'static str, Box<PolyFn>> = HashMap::new();
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
        Arc::new(Mutex::new(map))
    };
}

pub struct GlobalComponents;

impl GlobalComponents {
    pub fn unlock() -> MutexGuard<'static, HashMap<String, Component>> {
        COMPONENTS.lock().unwrap()
    }
}
pub struct GlobalFunctions;

impl GlobalFunctions {
    pub fn unlock() -> MutexGuard<'static, HashMap<&'static str, Box<PolyFn>>> {
        FUNCTIONS.lock().unwrap()
    }
}

pub struct Template {
    variables: Value,
}


impl Template {
    pub fn load(file: &str) -> Self {
        unimplemented!()
    }

    pub fn call_component(component: &Component) -> String {
        Codegen::call_component(component, None)
    }

    pub fn call_component_with_args(component: &Component, map: BTreeMap<String, Value>) -> String {
        Codegen::call_component(component, Some(map))
    }
}
