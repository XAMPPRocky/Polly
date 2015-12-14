use std::io::Error;
/// A simple abstraction to allow both text, and html elements to exist within the same vector.
pub trait Token {
    fn to_string(&self) -> Result<String, Error>;
}
