# Bad Ripgrep

![rust logo](https://rust-lang.org/logos/rust-logo-512x512.png)

### General Information:
ripgrep is a search tool that can recursively search directories for regex search patterns.
This implementation is trying to mimic its functionality with the goal of **learning the rust language**.

### Usage:
```bash
cargo run -- <search_word>
```

### Testing
Running unit tests:
```bash
cargo test
```

### Viewing the documentation
```bash
cargo doc --open
```

### Strategy:
The implementation will be using a fixed number of threads to find file paths for matching the target word and looking through the found paths concurrently.
