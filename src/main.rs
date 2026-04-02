use std::fs;
use std::path::{Path, PathBuf};

fn crawl_paths(starting_path: &Path) -> Vec<PathBuf> {
    let mut found_paths = Vec::new();
    let mut found_dirs = Vec::<PathBuf>::new();

    let entries = fs::read_dir(starting_path).unwrap();

    for entry in entries.flatten() {
        let curr_path = entry.path();

        if curr_path.is_dir() {
            found_dirs.push(curr_path);
        } else {
            found_paths.push(curr_path);
        }
    }

    for dir_path in found_dirs {
        found_paths.extend(crawl_paths(&dir_path));
    }

    found_paths
}

fn main() {
    let starting_path = Path::new("./");
    
    let paths = crawl_paths(starting_path);

    for path in paths {
        println!("{}", path.display());
    }

}
