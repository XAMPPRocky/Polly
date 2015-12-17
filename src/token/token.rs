use super::Element;

pub enum Token<'a> {
    PlainText(String),
    HTML(Element<'a>),
}
