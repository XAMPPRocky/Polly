#![deny(missing_docs,
        missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications)]
#[cfg(test)]
mod tests;
/// All the parser stuff
pub mod parser;
/// All the token stuff
pub mod token;
