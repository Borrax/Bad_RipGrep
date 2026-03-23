use std::env;

fn is_flag(arg: &str) -> bool {
    let Some(first_char) = arg.chars().next() else {
        panic!("Didn't receive a string argument")
    };
    first_char == '-' 
}

fn main() {
    let string_stuff = String::new();

    let args: Vec<String> = env::args().collect();
        for (i, arg) in args.iter().enumerate().skip(1) {
            println!("Argument {} {}", i, arg);
                if is_flag(arg) {
                    println!("Found a flag")
                }
        }
}
