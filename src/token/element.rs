use std::collections::HashMap;
use std::io::Error;

use super::Token;
/// Represents a HTML Element
pub struct Element<'a> {
    tag: &'a str,
    classes: Vec<&'a str>,
    attributes: HashMap<&'a str, &'a str>,
    children: Vec<Token<'a>>,
}

/// Provides errors related to modifying an [Element]()
#[derive(Debug)]
pub enum ElementError {
    AttributeExists,
    InvalidEncodingError,
}

impl<'a> Element<'a> {
    fn new(tag: &'a str) -> Self {
        Element {
            tag: tag,
            classes: Vec::new(),
            attributes: HashMap::new(),
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

    fn add_child(&mut self, child: Token<'a>) {
        self.children.push(child);
    }

    fn generate_text(&self) -> Result<String, ElementError> {
        use std::io::Write;
        let mut buffer: Vec<u8> = Vec::new();

        const VOID_ELEMENTS: [&'static str; 15] = ["area", "base", "br", "col", "embed", "hr",
                                                   "img", "input", "keygen", "link", "meta",
                                                   "param", "source", "track", "wbr"];

        let _ = write!(&mut buffer, "<{tag} ", tag = self.tag);
        let _ = write!(&mut buffer, "class=\"");
        for class in self.classes.iter() {
            let _ = write!(&mut buffer, "{class} ", class = class);
        }
        let _ = write!(&mut buffer, "\"");

        for (key, value) in self.attributes.iter() {
            let _ = write!(&mut buffer, "{key}=\"{value}\"", key = key, value = value);
        }
        let _ = write!(&mut buffer, ">");

        for child in self.children.iter() {
            let x = Token::html(Element::new(""));
            match *child {
                Token::PlainText(string) => buffer.append(&mut string.into_bytes()),
                Token::html(element) => {
                    if let Ok(string) = element.generate_text() {
                        buffer.append(&mut string.into_bytes())
                    }
                    let _ = write!(&mut buffer, " ");
                }
            }
        }

        let mut is_void = false;

        for element in VOID_ELEMENTS.iter() {
            if &self.tag == element {
                is_void = true;
                break;
            }
        }

        if is_void {
            let _ = write!(&mut buffer, ">");
        } else {
            let _ = write!(&mut buffer, "</{tag}>", tag = self.tag);
        }

        match String::from_utf8(buffer) {
            Ok(string) => Ok(string),
            Err(_) => Err(ElementError::InvalidEncodingError),
        }
    }
}
