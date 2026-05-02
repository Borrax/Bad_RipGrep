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

#[test]
fn test_more_matches() {
    let chars = ['i', 'p', 's', 'u', 'm'];
    let search_word: String = String::from_iter(chars);
    let search_re = Regex::new(&search_word).unwrap();

    let out = Arc::new(Mutex::new(Vec::new()));
    run_application(&search_re, out.clone());

    let result = String::from_utf8(out.lock().unwrap().to_vec()).unwrap();
    let lines: Vec<&str> = result.split("\n").collect();
    // last line is empty
    assert_eq!(lines.len(), 9); // including the generated from the benchmarks

    // last line is empty and for now the first lines are from benchmark file
    for line in &lines[5..lines.len() - 1] {
        assert!(line.contains(&search_word));
        
        assert_surrounding_words_count(line, &search_word);

    }
}   
