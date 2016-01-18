use std::collections::HashMap;
use std::result;

use compiler::AstResult;
use super::ast::{AstError, Token};


/// TODO
#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    tag: String,
    classes: Vec<String>,
    attributes: HashMap<String, String>,
    resource: Option<String>,
    children: Vec<AstResult>,
}


impl Element {
    /// TODO
    pub fn new(tag: String) -> Self {
        Element {
            tag: tag,
            classes: Vec::new(),
            attributes: HashMap::new(),
            resource: None,
            children: Vec::new(),
        }
    }
    /// Gets tag
    pub fn tag(&self) -> &String {
        &self.tag
    }

    /// Gets classes
    pub fn classes(&self) -> &Vec<String> {
        &self.classes
    }

    /// Gets attributes
    pub fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }

    /// Gets resource
    pub fn resource(&self) -> &Option<String> {
        &self.resource
    }

    /// Gets children
    pub fn children(&self) -> &Vec<AstResult> {
        &self.children
    }

    /// TODO
    pub fn add_resource(&mut self, resource: String) {
        self.resource = Some(resource);
    }

    fn add_child(&mut self, child: AstResult) {
        if self.resource == None {
            self.children.push(child);
        } else {
            panic!("An element can't have a resource file, and inline children. Child: {:?}",
                   child);
        }
    }
    /// TODO
    pub fn add_children(&mut self, children: &mut Vec<AstResult>) {
        self.children.append(children)
    }
    /// TODO
    pub fn add_class(&mut self, class: String) {
        self.classes.push(class);
    }
    /// TODO
    pub fn add_attribute(&mut self, key: String, value: String) {

        if key == "class" {
            self.classes.push(value);
        } else if let Some(prev_value) = self.attributes.insert(key.trim().to_owned(), value) {
            panic!("Attribute already exists! Old value: {:?}", prev_value);
        }
    }
    /// TODO
    pub fn add_text(&mut self, text: String) {
        self.children.push(Ok(Token::Text(text)));
    }
}
