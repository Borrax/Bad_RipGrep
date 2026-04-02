use std::fs;
use std::path::{Path, PathBuf};

fn crawl_paths(starting_path: &Path, dir_paths: &mut Vec<PathBuf>) -> Vec<PathBuf> {
    let mut found_paths = Vec::new();

    let entries = fs::read_dir(starting_path).unwrap();

    for entry in entries.flatten() {
        let curr_path = entry.path();

        if curr_path.is_dir() {
            dir_paths.push(curr_path);
        } else {
            found_paths.push(curr_path);
        }
    }

    found_paths
}

fn main() {
    let starting_path = Path::new("./");
    let mut found_dirs = Vec::<PathBuf>::new();
    
    crawl_paths(starting_path, &mut found_dirs);

}
