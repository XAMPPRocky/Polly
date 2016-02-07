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
            name: name.trim().to_owned(),
            args: Vec::new(),
            ast: Vec::new(),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
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

    pub fn add_arg_value(&mut self, value: String) {
        self.args.push(ArgKey::Json(value));
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
            name: name.trim().to_owned(),
            values: Vec::new(),
        }
    }

    pub fn from_component(component: Component) -> Self {
        ComponentCall {
            name: component.name(),
            values: component.args(),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn values(&self) -> Vec<ArgKey> {
        self.values.clone()
    }

    pub fn add_value(&mut self, name: String) {
        self.values.push(ArgKey::Json(name));
    }
}
