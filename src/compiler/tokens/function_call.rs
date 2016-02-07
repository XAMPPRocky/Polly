use std::collections::BTreeMap;
use super::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FunctionCall {
    identifier: String,
    arguments: BTreeMap<String, ArgKey>,
}


impl FunctionCall {
    pub fn new(identifier: String) -> Self {
        FunctionCall { identifier: identifier.trim().to_owned(), ..Self::default() }
    }

    pub fn identifier(&self) -> String {
        self.identifier.clone()
    }
    pub fn args(&self) -> &BTreeMap<String, ArgKey> {
        &self.arguments
    }
    pub fn add_value_arg(&mut self, key: String, value: String) {
        self.arguments.insert(key.trim().to_owned(), ArgKey::Json(value));
    }

    pub fn add_component_arg(&mut self, key: String, value: String) {
        self.arguments.insert(key.trim().to_owned(), ArgKey::Comp(value));
    }
}
