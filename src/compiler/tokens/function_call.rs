use std::convert::Into;
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

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn args(&self) -> &BTreeMap<String, ArgKey> {
        &self.arguments
    }

    pub fn add_value_arg<SK: AsRef<str>, SV: Into<String>>(&mut self, key: SK, value: SV) {
        self.arguments.insert(key.as_ref().trim().into(), ArgKey::Json(value.into()));
    }

    pub fn add_component_arg<SK: AsRef<str>, SV: Into<String>>(&mut self, key: SK, value: SV) {
        self.arguments.insert(key.as_ref().trim().into(), ArgKey::Comp(value.into()));
    }
}
