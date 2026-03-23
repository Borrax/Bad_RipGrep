use std::env;

fn main() {
    let string_stuff = String::new();

    let args: Vec<String> = env::args().collect();
        for (i, arg) in args.iter().enumerate().skip(1) {
            println!("Argument {} {}", i, arg);
            if let Some(first_char) = arg.chars().next() && first_char == '-' {
                println!("Found a flag");
            }
        }
}
