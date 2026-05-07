use std::sync::{Arc, Mutex};
use regex::Regex;
use bad_ripgrep::run_application;


const MAX_WORDS_AROUND_MATCH: usize = 5;

fn assert_surrounding_words_count(text: &str, search_word: &str) {
    let splitted_text: Vec<&str> = text.split(search_word).collect();
    let words_before = splitted_text[0].split_whitespace().collect::<Vec<&str>>();
    assert!(words_before.len() <= MAX_WORDS_AROUND_MATCH + 1); // +1 because it will
                                                               // include the line index

    let words_after = splitted_text[1].split_whitespace().collect::<Vec<&str>>();
    assert!(words_after.len() <= MAX_WORDS_AROUND_MATCH);
}

fn run_and_get_result(search_re: &Regex) -> String {
    let out = Arc::new(Mutex::new(Vec::new()));
    run_application(search_re, out.clone());

    return String::from_utf8(out.lock().unwrap().to_vec()).unwrap();
}

#[test]
fn test_more_matches() {
    let chars = ['i', 'p', 's', 'u', 'm'];
    let search_word: String = String::from_iter(chars);
    let search_re = Regex::new(&search_word).unwrap();

    let result = run_and_get_result(&search_re);
    let lines: Vec<&str> = result.split("\n").collect();
    // last line is empty
    assert_eq!(lines.len(), 9); // including the generated from the benchmarks

    // last line is empty
    for line in &lines[5..lines.len() - 1] {
        println!("{}", line);
        assert!(line.contains(&search_word));
        
        assert_surrounding_words_count(line, &search_word);

    }
}   

#[test]
fn test_non_existent_word() {
    let chars = ['b', 'u', 'l', 'b', 'a', 's', 'a', 'u', 'r'];
    let search_word = String::from_iter(chars);
    let search_re = Regex::new(&search_word).unwrap();

    let result = run_and_get_result(&search_re);

    assert_eq!("", result);
}
