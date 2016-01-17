#[macro_use]
extern crate clap;
extern crate poly;

use clap::App;
use poly::codegen::Codegen;
use std::fs::{File, metadata};

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let paths = matches.values_of("input").unwrap();

    for path in paths {
        if metadata(path)
               .ok()
               .expect("Couldn't find file, please make sure your path is correct.")
               .is_file() {
            use std::io::Read;
            let mut file = File::open(path).ok().expect("This file couldn't be opened");
            let mut contents = String::new();
            file.read_to_string(&mut contents).ok().expect("Couldn't write to buffer");
            let html = Codegen::codegen(&*contents, path);

            if let Some(path) = matches.value_of("file") {
                use std::io::Write;
                let mut file = File::create(path)
                                   .ok()
                                   .expect("Couldn't create file at destination");
                file.write_all(&html.into_bytes()).ok().expect("Couldn't write to file");
            } else {
                println!("{}", html);
            }
        }
    }
}
