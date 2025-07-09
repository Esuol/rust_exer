fn main() {
    run_string();

    let s: String = String::from("hello world");

    let word = run_slice(&s);

    println!("current_word: {word}");

    let s2: &str = "rust code";

    let word = run_slice(s2);

    println!("current_word: {word}");
}

fn run_string() {
    let mut s = String::from("hello");

    s.push_str(", world!");

    println!("{}", s);
}

fn run_slice(s: &str) -> &str {
    let bytes = s.as_bytes();

    for (i, &item) in bytes.iter().enumerate() {
        if item == b' ' {
            return &s[..i];
        }
    }

    &s[..]
}
