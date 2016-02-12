use std::convert::Into;
use super::ArgKey;
use compiler::AstResult;

#[derive(Clone, Debug, PartialEq)]
pub struct Component {
    name: String,
    args: Vec<ArgKey>,
    ast: Vec<AstResult>,
}

impl Component {
    pub fn new(name: String) -> Self {
        Component {
            name: name.trim().into(),
            args: Vec::new(),
            ast: Vec::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn args(&self) -> Vec<ArgKey> {
        self.args.clone()
    }

    pub fn number_of_args(&self) -> usize {
        self.args().len()
    }

    pub fn ast(&self) -> Vec<AstResult> {
        self.ast.clone()
    }

    pub fn add_arg_value<V: Into<String>>(&mut self, value: V) {
        self.args.push(ArgKey::Json(value.into()));
    }
    pub fn add_children(&mut self, children: &mut Vec<AstResult>) {
        self.ast.append(children);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ComponentCall {
    name: String,
    values: Vec<ArgKey>,
}

impl ComponentCall {
    pub fn new(name: String) -> Self {
        ComponentCall {
            name: name.trim().into(),
            values: Vec::new(),
        }
    }

    pub fn from_component(component: Component) -> Self {
        ComponentCall {
            name: component.name().into(),
            values: component.args(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn values(&self) -> &[ArgKey] {
        &self.values[..]
    }

    pub fn add_value<V: Into<String>>(&mut self, name: V) {
        self.values.push(ArgKey::Json(name.into()));
    }
}
