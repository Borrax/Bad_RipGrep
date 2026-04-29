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
use bad_ripgrep::run_application;

fn main() {
    run_application();
}
