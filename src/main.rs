use std::collections::VecDeque;
use std::fs;
use std::io::{BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

type GlobalPathQueue = Arc<Mutex<VecDeque<PathBuf>>>;

fn crawl_paths(starting_path: &Path, paths_mutex_queue: &GlobalPathQueue) {
    let mut found_dirs = Vec::<PathBuf>::new();
    let mut paths_queue = paths_mutex_queue.lock().unwrap();

    let entries = fs::read_dir(starting_path).unwrap();

    for entry in entries.flatten() {
        let curr_path = entry.path();

        if curr_path.is_dir() {
            found_dirs.push(curr_path);
        } else {
            paths_queue.push_back(curr_path);
        }
    }

    for dir_path in found_dirs {
        crawl_paths(&dir_path, paths_mutex_queue);
    }
}

fn look_for_match_in_file(path: &Path, search_str: &str) {
    let file = fs::File::open(path).expect("Could not open the file");
    let reader = BufReader::new(file);
    let max_line_length = 60;

    for (index, line) in reader.lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.contains(search_str) {
            println!("{}: {:.max_line_length$}", index + 1, line);
        }
    }
}

fn main() {
    let starting_path = Path::new("./");
    let mutex_queue: GlobalPathQueue = Arc::new(Mutex::new(VecDeque::new()));

    let queue_clone = Arc::clone(&mutex_queue);
    crawl_paths(starting_path, &queue_clone);

    // while let Some(val) = queue_clone.pop_front() {
    //     println!("{}", val.display());
    // }
}
