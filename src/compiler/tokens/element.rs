use std::collections::HashMap;

use compiler::AstResult;
use super::ComponentCall;

/// The AST representation of a HTML element.
#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    tag: String,
    classes: Vec<String>,
    attributes: HashMap<String, String>,
    resource: Option<ComponentCall>,
    children: Vec<AstResult>,
}

impl Element {
    pub fn new(tag: String) -> Self {
        Element {
            tag: tag,
            classes: Vec::new(),
            attributes: HashMap::new(),
            resource: None,
            children: Vec::new(),
        }
    }

    pub fn tag(&self) -> &String {
        &self.tag
    }

    pub fn classes(&self) -> &Vec<String> {
        &self.classes
    }

    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    pub fn resource(&self) -> &Option<ComponentCall> {
        &self.resource
    }

    pub fn children(&self) -> &Vec<AstResult> {
        &self.children
    }

    pub fn add_resource(&mut self, resource: ComponentCall) {
        self.resource = Some(resource);
    }

    pub fn add_children(&mut self, children: &mut Vec<AstResult>) {
        self.children.append(children)
    }

    pub fn add_class(&mut self, class: String) {
        self.classes.push(class);
    }

    pub fn add_attribute(&mut self, key: String, value: String) {
        if key == "class" {
            self.classes.push(value);
        } else {
            self.attributes.insert(key.trim().to_owned(), value);
        }
    }
}
