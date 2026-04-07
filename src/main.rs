use std::collections::VecDeque;
use std::{fs, num};
use std::io::{BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;

type GlobalPathQueue = Arc<Mutex<VecDeque<PathBuf>>>;

#[derive(Clone, Debug)]
enum AppEvent {
    NewDirPathPushed,
    Shutdown,
}

fn crawl_paths(starting_path: &Path, paths_mutex_queue: &GlobalPathQueue) {
    let mut found_dirs = Vec::<PathBuf>::new();

    let entries = fs::read_dir(starting_path).unwrap();

    for entry in entries.flatten() {
        let curr_path = entry.path();

        if curr_path.is_dir() {
            found_dirs.push(curr_path);
        } else {
            let mut paths_queue = paths_mutex_queue.lock().unwrap();
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

fn search_worker(paths_mutex_queue: GlobalPathQueue) {
    loop {
        let path = {
            let mut q = paths_mutex_queue.lock().unwrap();
            q.pop_front()
        };

        let path = match path {
            Some(val) => val,
            None => break,
        };

        
        look_for_match_in_file(&path, "ipsum");
    }

}

fn paths_worker(paths_queue: GlobalPathQueue, dir_paths_queue: GlobalPathQueue, 
    still_finding_paths: Arc<Vec<AtomicBool>>, idx: u32) {
    loop {
        if still_finding_paths.iter().all(|b| !b.load(Ordering::Relaxed)) {
            break;
        }

        let starting_path = {
            let mut q = dir_paths_queue.lock().unwrap();
            q.pop_front()
        };

        let starting_path = match starting_path {
            Some(val) => {
                still_finding_paths[idx].store(true, Ordering::Relaxed);
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
    }
}

fn main() {
    let num_threads = 6;
    let starting_path = PathBuf::from("./");
    let mutex_queue_paths: GlobalPathQueue = Arc::new(Mutex::new(VecDeque::new()));
    let mutex_queue_dirs: GlobalPathQueue = Arc::new(Mutex::new(VecDeque::from([starting_path])));
    // boolean array to check if a thread is still searching for paths
    let still_finding_paths = Arc::new(
        (0..num_threads).map(|_| AtomicBool::new(false)).collect()
    );


    let mut search_handles = Vec::new();
    let mut path_handles = Vec::new();

    // let queue_clone = Arc::clone(&mutex_queue_paths);
    // crawl_paths(starting_path, &queue_clone);

    for idx in 0..num_threads {
        let path_q = Arc::clone(&mutex_queue_paths);
        let dir_path_q = Arc::clone(&mutex_queue_dirs);
        let still_finding_paths_clone = Arc::clone(&still_finding_paths);

        let handle = thread::spawn(
            move || paths_worker(path_q, dir_path_q, still_finding_paths_clone, idx)
        );

        path_handles.push(handle);
    }

    for handle in path_handles {
        handle.join().unwrap();
    }

    for _ in 0..num_threads {
        let q = Arc::clone(&mutex_queue_paths);

        let handle = thread::spawn(move || search_worker(q));
        search_handles.push(handle);
    }

    for handle in search_handles {
        handle.join().unwrap();
    }
}
