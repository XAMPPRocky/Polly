use std::collections::HashMap;

use serde_json::Value;

use compiler::tokens::Args;

pub type PolyFn = Fn(Vec<Args>) -> String;


pub struct Template<'a> {
    variables: Value,
    functions: HashMap<&'a str, Box<PolyFn>>,
}


impl<'a> Template<'a> {
    pub fn load(file: &str) -> Self {
        unimplemented!()
    }
}
