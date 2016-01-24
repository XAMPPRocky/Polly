use std::collections::HashMap;
use std::sync::Mutex;

use serde_json::Value;

use compiler::tokens::{Args, Component};

pub type PolyFn = Fn(Vec<Args>) -> String;

lazy_static! {
    pub static ref COMPONENTS: Mutex<HashMap<String, Component>> = {
        Mutex::new(HashMap::new())
    };
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
