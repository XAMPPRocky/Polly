use std::collections::HashMap;
use std::io::Error;

use super::token::Token;
/// Represents a HTML Element
#[derive(Debug)]
pub struct Element<'a> {
    tag: &'a str,
    classes: Vec<&'a str>,
    attributes: HashMap<&'a str, &'a str>,
    text: String,
    children: Vec<Box<Token + 'a>>,
}

/// Provides errors related to modifying an [Element]()
#[derive(Debug)]
pub enum ElementError {
    AttributeExists,
}

impl<'a> Element<'a> {
    fn new(tag: &'a str) -> Self {
        Element {
            tag: tag,
            classes: Vec::new(),
            attributes: HashMap::new(),
            text: String::new(),
            children: Vec::new(),
        }
    }

    fn add_class(&mut self, class: &'a str) {
        self.classes.push(class)
    }

    fn add_attribute(&mut self, attribute: &'a str, value: &'a str) -> Result<(), ElementError> {
        match self.attributes.insert(attribute, value) {
            Some(previous_value) => Err(ElementError::AttributeExists),
            None => Ok(()),
        }
    }

    fn add_text(&mut self, text: String) {
        self.text.push_str(&*text);
    }

    fn add_child<T: Token + 'a>(&mut self, child: T) {
        self.children.push(Box::new(child));
    }
}


impl<'a> Token for Element<'a> {
    fn to_string(&self) -> Result<String, Error> {
        use std::io::Write;

        let mut s = Vec::new();

        try!(write!(&mut s, "<{tag} ", tag = self.tag));
        try!(write!(&mut s, ""));
        Ok(String::from_utf8(s).unwrap())
    }
}
