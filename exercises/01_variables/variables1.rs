use std::io;

fn main() {
    let x = 5;

    expect_value();

    stdin_value();

    println!("x has the value {x}");
}

fn expect_value() {
    let guess: u32 = "42".parse().expect("Not a number");

    println!("guess: {guess}");
}

fn stdin_value() {
    let a = [1, 2, 3, 4, 5];

    println!("Please enter an array index.");

    let mut index = String::new();

    io::stdin()
        .read_line(&mut index)
        .expect("Failed to read line");

    let index: usize = index
        .trim()
        .parse()
        .expect("Index entered was not a number");

    let element = a[index];

    println!("The value of the element at index {index} is: {element}");
}


