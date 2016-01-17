#![deny(trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications)]
//! The Poly parser.

#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;

mod args;
mod ast;
/// All the codgen stuff
pub mod codegen;
mod consts;
mod element;
mod lexeme;
mod lexer;
mod operator;
mod parser;
mod template;
