//!  # Bad Rip Grep
//!  Crawls recursively through directories and prints out all the found matches of a word/regex
//!  expression 
//!
//!  # Usage:
//!  ```
//!  cargo run -- <word/regex>
//!
//!  ```
//!  or
//!  ```
//!  <built_target> <word/regex>
//!  ```
use regex::Regex;
use std::sync::{Arc, Mutex};

use bad_ripgrep::run_application;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let search_word = match args.get(1) {
        Some(s) => s.clone(),
        None => {
            panic!("No argument provided")
        }
    };

    let re = match Regex::new(&search_word) {
        Ok(m) => m,
        Err(_) => panic!("Word expression can't be matched!")
    };

    run_application(&re, Arc::new(Mutex::new(std::io::stdout())));
}
