#[macro_use]
extern crate clap;
extern crate polly;
extern crate serde_json;

use std::collections::BTreeMap;
use std::fs::{File, metadata};
use std::io::{Read, Write};

use clap::App;
use polly::Template;
use serde_json::Value;

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let paths = matches.values_of("input").unwrap();

    for path in paths {
        let path_metadata = metadata(path)
                                .ok()
                                .expect("Couldn't find file, please make sure your path is \
                                         correct.");
        if path_metadata.is_file() {
            let mut file = File::open(path).ok().expect("This file couldn't be opened");
            let mut contents = String::new();
            file.read_to_string(&mut contents).ok().expect("Couldn't write to buffer");
            let lang = match matches.value_of("lang") {
                Some(lang) => lang,
                _ => "en",
            };

            let json = if let Some(path) = matches.value_of("json") {
                let contents = {
                    let mut file = File::open(path).expect("JSON File not found");
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).expect("Couldn't JSON read file.");
                    contents
                };
                match serde_json::from_str(&*contents).expect("Wasn't valid JSON") {
                    Value::Object(object) => object,
                    _ => panic!("Wasn't a JSON object"),
                }
            } else {
                BTreeMap::new()
            };

            let html = if matches.is_present("no-locales") {
                Template::load(path).unwrap().json(json).no_locales().render(lang).unwrap()
            } else {
                Template::load(path).unwrap().json(json).render(lang).unwrap()
            };

            if let Some(path) = matches.value_of("file") {
                let mut file = File::create(path)
                                   .ok()
                                   .expect("Couldn't create file at destination");
                file.write_all(&html.into_bytes()).ok().expect("Couldn't write to file");
            } else {
                println!("{}", html);
            }
        } else {
            panic!("Path provided wasn't a file: {}", path);
        }
    }
}
