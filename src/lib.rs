#![deny(trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications)]
//! The Poly parser.


/// All the lexer stuff
pub mod lexer;
/// All the parser stuff
pub mod parser;
/// All the codgen stuff
pub mod codegen;
/// All the token stuff
pub mod element;
/// All the operator stuff
pub mod operator;
/// All the operator stuff
pub mod consts;
/// All the operator stuff
pub mod ast;
