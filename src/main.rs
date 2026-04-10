use std::collections::VecDeque;
use std::{fs};
use std::io::{BufReader, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;

type GlobalPathQueue = Arc<Mutex<VecDeque<PathBuf>>>;

fn look_for_match_in_file<W: Write>(path: &Path, search_str: &str,
    out: &mut W) -> std::io::Result<()>{
    let file = fs::File::open(path).expect("Could not open the file");
    let reader = BufReader::new(file);
    let max_line_length = 60;

    for (index, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.contains(search_str) {
            // println!("{}: {:.max_line_length$}", index + 1, line);
            writeln!(out, "{}: {:.max_line_length$}", index + 1, line)?;
        }
    }
   Ok(()) 
}

fn search_worker(paths_mutex_queue: GlobalPathQueue, dir_paths_queue: GlobalPathQueue,
    still_finding_paths: Arc<Vec<AtomicBool>>, search_str: String) {
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

        
        look_for_match_in_file(&path, &search_str, &mut std::io::stdout());
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

        let search_str = search_word.clone();
        let handle_search = thread::spawn(move || search_worker(path_q2, dir_path_q2, still_finding_paths_clone2, search_str));

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

    #[test]
    fn test_single_match() {
        let path: &Path = Path::new("./test.txt");

        let chars = ['c', 'u', 'p', 'i'];
        let search_word: String = String::from_iter(chars);
        look_for_match_in_file(path, &search_word);
    }   

    #[test]
    fn test_more_matches() {
        let path: &Path = Path::new("./test.txt");

        let chars = ['i', 'p', 's', 'u', 'm'];
        let search_word: String = String::from_iter(chars);
        look_for_match_in_file(path, &search_word);
    }   

    #[test]
    fn test_non_existent_word() {
        let path: &Path = Path::new("./test.txt");

        let chars = ['b', 'u', 'l', 'b', 'a', 's', 'a', 'u', 'r'];
        let search_word = String::from_iter(chars);
        look_for_match_in_file(path, &search_word);
    }

    #[test]
    fn test_empty_string_seatch() {
        let path: &Path = Path::new("./test.txt");

        let search_word = "".to_string();
        look_for_match_in_file(path, &search_word);
    }
}
