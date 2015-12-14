use std::io::Error;

use super::token::Token;

impl Token for String {
    fn to_string(&self) -> Result<String, Error> {
        Ok(self.clone())
    }
}
