#![deny(trivial_casts, trivial_numeric_casts,
        unstable_features,
        unused_import_braces, unused_qualifications)]

//! The Poly parser.

#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;

/// All the codgen stuff
pub mod compiler;
pub mod template;
