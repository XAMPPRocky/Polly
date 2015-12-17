use poly::parser::*;

#[test]
fn take_single_char() {
    let text = "A";
    let parser = Parser::new(text.clone());
    assert_eq!(Some('A'), parser.take());
    assert!(parser.eof());
}
#[test]
fn peek_single_char() {
    let parser = Parser::new("Hello World!");
    assert_eq!(Some('H'), parser.peek());
}
