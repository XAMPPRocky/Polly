use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use serde_json::Value;

use compiler::tokens::{Args, Component};

pub type PolyFn = Fn(Vec<Args>) -> String;

lazy_static! {
    static ref COMPONENTS: Mutex<HashMap<String, Component>> = {
        Mutex::new(HashMap::new())
    };
    static ref FUNCTIONS: Mutex<HashMap<&'static str, Args>> = {
        Mutex::new(HashMap::new())
    };
}

pub struct GlobalComponents;

impl GlobalComponents {
    // pub fn get(name: &str) -> &Option<&Component> {
    //     &GlobalComponents::unlock().get(name)
    // }

    // pub fn insert(key: String, value: Component) {
    //     GlobalComponents::unlock().insert(key, value);
    // }

    pub fn unlock() -> MutexGuard<'static, HashMap<String, Component>> {
        COMPONENTS.lock().unwrap()
    }
}
pub struct GlobalFunctions;

impl GlobalFunctions {
    // pub fn get(name: &str) -> &Option<&Component> {
    //     &GlobalComponents::unlock().get(name)
    // }

    // pub fn insert(key: String, value: Component) {
    //     GlobalComponents::unlock().insert(key, value);
    // }

    pub fn unlock() -> MutexGuard<'static, HashMap<&'static str, Args>> {
        FUNCTIONS.lock().unwrap()
    }
}

pub struct Template<'a> {
    variables: Value,
    functions: HashMap<&'a str, Box<PolyFn>>,
}


impl<'a> Template<'a> {
    pub fn load(file: &str) -> Self {
        unimplemented!()
    }
}
