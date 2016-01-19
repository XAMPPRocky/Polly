use compiler::Codegen;
use serde_json::Value;
use super::Args;
use super::Token;
use compiler::AstResult;

#[derive(Clone, Debug, PartialEq)]
pub struct Component {
    name: String,
    args: Vec<Args>,
    ast: Vec<AstResult>,
}

impl Component {
    pub fn new(name: String) -> Self {
        Component {
            name: name,
            args: Vec::new(),
            ast: Vec::new(),
        }
    }

    pub fn add_arg_value(&mut self, value: String) {
        self.args.push(Args::Generic(value));
    }

    pub fn add_arg_component(&mut self, value: String) {
        self.args.push(Args::Component(value));
    }

    pub fn add_children(&mut self, children: &mut Vec<AstResult>) {
        self.ast.append(children);
    }

    pub fn render(&self) -> String {
        Codegen::from_component(self)
    }
}
