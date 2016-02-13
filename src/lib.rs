#![deny(trivial_casts, trivial_numeric_casts,
        unused_import_braces, unused_qualifications)]
//! The Poly parser.

#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
mod compiler;
mod template;


pub use template::{PolyFn, Template};
pub use compiler::ArgValue;
