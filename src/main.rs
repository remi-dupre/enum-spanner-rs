mod automaton;
mod mapping;
mod matrix;
mod regex;
mod settings;
mod tools;

extern crate clap;
extern crate indicatif;
extern crate rand;
extern crate regex_syntax;

use std::fs::File;
use std::io::prelude::*;
use std::io::stdin;

use clap::{App, Arg};

fn main() {
    //  ____
    // |  _ \ __ _ _ __ ___  ___ _ __
    // | |_) / _` | '__/ __|/ _ \ '__|
    // |  __/ (_| | |  \__ \  __/ |
    // |_|   \__,_|_|  |___/\___|_|
    //
    let matches = App::new("Enumerate matchings")
        .version("0.1")
        .author("Rémi Dupré <remi.dupre@ens-paris-saclay.fr>")
        .about("Enumerate all matches of a regular expression on a text.")
        .arg(
            Arg::with_name("regex")
                .help("The pattern to look for.")
                .required(true),
        )
        .arg(
            Arg::with_name("file")
                .help("The file to be read, if none is specified, STDIN is used."),
        )
        .arg(
            Arg::with_name("bytes_offset")
                .short("b")
                .long("bytes-offset")
                .help("Print the 0-based offset of each matching part and groups."),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Display the number of matches instead."),
        )
        .get_matches();

    // Extract parameters
    let show_offset = matches.is_present("bytes_offset");
    let count = matches.is_present("count");
    let regex = matches.value_of("regex").unwrap(); // Safe unwrap

    // Read the text
    let mut text = String::new();
    match matches.value_of("file") {
        Some(filename) => {
            let mut file = File::open(filename).unwrap();
            file.read_to_string(&mut text).unwrap()
        }
        None => stdin().read_to_string(&mut text).unwrap(),
    };

    // Remove trailing newlines
    while text.as_bytes().last() == Some(&b'\n') {
        text.pop();
    }

    //  __  __       _       _
    // |  \/  | __ _| |_ ___| |__
    // | |\/| |/ _` | __/ __| '_ \
    // | |  | | (_| | || (__| | | |
    // |_|  |_|\__,_|\__\___|_| |_|
    //
    let regex = regex::compile(regex);
    regex
        .render("automaton.dot")
        .expect("Could not create the dotfile.");

    let compiled_matches = mapping::IndexedDag::compile(regex, text.clone());

    if count {
        let count: u64 = compiled_matches.iter().map(|_| 1).sum();
        println!("{}", count)
    } else {
        for (count, mapping) in compiled_matches.iter().enumerate() {
            print!("{} -", count);

            for (name, range) in mapping.iter_groups() {
                if show_offset {
                    print!(" {}:{},{}", name, range.start, range.end);
                } else {
                    print!(" {}:\"{}\"", name, &text[range]);
                }
            }

            println!();
        }
    }
}
