use std::collections::VecDeque;
use std::fs;
use std::io::{BufReader, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use regex::Regex;

type GlobalPathQueue = Arc<Mutex<VecDeque<PathBuf>>>;

/// Matches a regex expression through a file
///
/// # Arguments
/// * `path` - The path to the file relative to where the command was ran from
/// * `re` - The regex expression taht needs to be matched
/// * `out` - Output buffer the results are written to
fn look_for_match_in_file<W: Write>(path: &Path, re: &Regex,
    out: &mut W) {
    let file = fs::File::open(path).expect("Could not open the file");
    let reader = BufReader::new(file);
    let max_words_around_match = 5;

    for (index, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if let Some(m) = re.find(&line) {
            let bytes_before = &line[..m.start()];
            let bytes_after = &line[m.end()..];

            let words_before: String = bytes_before.split_whitespace().rev().take(max_words_around_match)
                .collect::<Vec<&str>>().join(" ");
            let words_after: String = bytes_after.split_whitespace().take(max_words_around_match)
                .collect::<Vec<&str>>().join(" ");

            let _ = writeln!(out, "{}: {} {} {}", index + 1, words_before, m.as_str(), words_after);
        }
    }

}

/// Pops file paths from the queue and looks for a match in the file
///
/// # Arguments
/// * `paths_mutex_queue` - Queue containing file paths
/// * `dir_paths_queue` - A queue containing all the untraversed directories
/// * `still_finding_paths` - Bool vector for every file path finding thread if they are still active
/// * `re` - The target regex expression that needs to be matched
fn search_worker(paths_mutex_queue: GlobalPathQueue, dir_paths_queue: GlobalPathQueue,
    still_finding_paths: Arc<Vec<AtomicBool>>, re: Regex) {
    loop {
        let no_new_paths_coming = still_finding_paths.iter().all(|b| !b.load(Ordering::Relaxed));

        if no_new_paths_coming && dir_paths_queue.lock().unwrap().is_empty() {
            break;
        }

        let path = {
            let mut q = paths_mutex_queue.lock().unwrap();
            q.pop_front()
        };

        let path = match path {
            Some(val) => val,
            None => continue,
        };

        
        look_for_match_in_file(&path, &re, &mut std::io::stdout());
    }

}

fn paths_worker(paths_queue: GlobalPathQueue, dir_paths_queue: GlobalPathQueue, 
    still_finding_paths: Arc<Vec<AtomicBool>>, thread_idx: usize) {
    loop {
        let no_new_paths_coming = still_finding_paths.iter().all(|b| !b.load(Ordering::Relaxed));

        if no_new_paths_coming && dir_paths_queue.lock().unwrap().is_empty() {
            break;
        }

        let starting_path = {
            let mut q = dir_paths_queue.lock().unwrap();
            q.pop_front()
        };

        let starting_path = match starting_path {
            Some(val) => {
                still_finding_paths[thread_idx].store(true, Ordering::Relaxed);
                val
            },
            None => continue,
        };

        let entries = fs::read_dir(starting_path).unwrap();

        for entry in entries.flatten() {
            let curr_path = entry.path();

            if curr_path.is_dir() {
                let mut dir_queue = dir_paths_queue.lock().unwrap();
                dir_queue.push_back(curr_path);
            } else {
                let mut path_queue = paths_queue.lock().unwrap();
                path_queue.push_back(curr_path);
            }
        }


        still_finding_paths[thread_idx].store(false, Ordering::Relaxed);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let search_word = match args.get(1) {
        Some(s) => s.clone(),
        None => {
            panic!("No argument provided")
        }
    };

    let num_threads = 6;
    let starting_path = PathBuf::from("./");
    let mutex_queue_paths: GlobalPathQueue = Arc::new(Mutex::new(VecDeque::new()));
    let mutex_queue_dirs: GlobalPathQueue = Arc::new(Mutex::new(VecDeque::from([starting_path])));
    // boolean array to check if a thread is still searching for paths
    let still_finding_paths = Arc::new(
        (0..num_threads).map(|_| AtomicBool::new(false)).collect()
    );

    let mut handles = Vec::new();

    let re = match Regex::new(&search_word) {
        Ok(m) => m,
        Err(_) => panic!("Word expression can't be matched!")
    };

    // Spawn a fixed number of threads that would concurrently find paths
    // and search files for a match
    for idx in 0..num_threads {
        let path_q = Arc::clone(&mutex_queue_paths);
        let dir_path_q = Arc::clone(&mutex_queue_dirs);
        let still_finding_paths_clone = Arc::clone(&still_finding_paths);

        let path_q2 = Arc::clone(&mutex_queue_paths);
        let dir_path_q2 = Arc::clone(&mutex_queue_dirs);
        let still_finding_paths_clone2 = Arc::clone(&still_finding_paths);

        let path_handle = thread::spawn(
            move || paths_worker(path_q, dir_path_q, still_finding_paths_clone, idx)
        );

        let cloned_re = re.clone();
        let handle_search = thread::spawn(move || search_worker(path_q2, dir_path_q2, still_finding_paths_clone2, cloned_re));

        handles.push(handle_search);
        handles.push(path_handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[cfg(test)]
mod test_look_for_match_in_file {
    use super::*;
    use std::sync::LazyLock;

    static PATH: LazyLock<&Path> = LazyLock::new(|| Path::new("./test.txt"));
    const MAX_WORDS_AROUND_MATCH: usize = 5;

    fn run_target(re: &Regex) -> String {
        let mut out = Vec::new();
        look_for_match_in_file(&PATH, re, &mut out);
        String::from_utf8(out).unwrap()
    }

    fn assert_surrounding_words_count(text: &str, search_word: &str) {
        let splitted_text: Vec<&str> = text.split(search_word).collect();
        let words_before = splitted_text[0].split_whitespace().collect::<Vec<&str>>();
        assert!(words_before.len() <= MAX_WORDS_AROUND_MATCH + 1); // +1 because it will
                                                                   // include the line index

        let words_after = splitted_text[1].split_whitespace().collect::<Vec<&str>>();
        assert!(words_after.len() <= MAX_WORDS_AROUND_MATCH);
    }

    #[test]
    fn test_single_match() {
        let chars = ['c', 'u', 'p', 'i'];
        let search_word: String = String::from_iter(chars);
        let search_re = Regex::new(&search_word).unwrap();
        // look_for_match_in_file(path, &search_word);
        let result = run_target(&search_re);

        assert!(result.contains(&search_word));

        assert_surrounding_words_count(&result, &search_word);
    }   

    #[test]
    fn test_more_matches() {
        let chars = ['i', 'p', 's', 'u', 'm'];
        let search_word: String = String::from_iter(chars);
        let search_re = Regex::new(&search_word).unwrap();
        let result = run_target(&search_re);

        let lines: Vec<&str> = result.split("\n").collect();
        // last line is empty
        assert_eq!(lines.len(), 4);

        // last line is empty
        for line in &lines[0..lines.len() - 1] {
            assert!(line.contains(&search_word));
            
            assert_surrounding_words_count(line, &search_word);

        }
    }   

    #[test]
    fn test_bigger_search_string() {
        let chars = ['d', 'o', 'l', 'o', 'r', ' ', 's', 'i', 't', ' ', 'a', 'm', 'e', 't'];
        let search_word: String = String::from_iter(chars);
        let search_re = Regex::new(&search_word).unwrap();
        let result = run_target(&search_re);

        let lines: Vec<&str> = result.split("\n").collect();
        // last line is empty
        assert_eq!(lines.len(), 2);

        // last line is empty
        for line in &lines[0..lines.len() - 1] {
            assert!(line.contains(&search_word));
            
            assert_surrounding_words_count(line, &search_word);
        }
    }   

    #[test]
    fn test_regex_expression() {
        let search_re = Regex::new(r"\bl\w+m\b").unwrap();
        let result = run_target(&search_re);

        let lines: Vec<&str> = result.split("\n").collect();
        // last line is empty
        assert_eq!(lines.len(), 3);

        // last line is empty
        for line in &lines[0..lines.len() - 1] {
            assert!(search_re.find(line).is_some());
        }
    }   

    #[test]
    fn test_non_existent_word() {
        let chars = ['b', 'u', 'l', 'b', 'a', 's', 'a', 'u', 'r'];
        let search_word = String::from_iter(chars);
        let search_re = Regex::new(&search_word).unwrap();

        let result = run_target(&search_re);

        assert_eq!("", result);
    }

    #[test]
    fn test_empty_string_search() {
        let search_word = "".to_string();
        let search_re = Regex::new(&search_word).unwrap();

        let result = run_target(&search_re);
        
        let lines: Vec<&str> = result.split("\n").collect();
        // Assert no panic and a bunch of matches
        assert!(lines.len() > 1);
    }

    #[test]
    fn test_non_existent_path_panics() {
        let chars = ['i', 'p', 's', 'u', 'm'];
        let search_word: String = String::from_iter(chars);
        let search_re = Regex::new(&search_word).unwrap();
        let path: &Path = Path::new("./test/path");

        let mut out = Vec::new();
        let result = std::panic::catch_unwind(move || {
            look_for_match_in_file(path, &search_re, &mut out)
        });

        assert!(result.is_err());
    }
}
