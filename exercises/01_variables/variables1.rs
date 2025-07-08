fn main() {
    let x = 5;

    expect_value();

    println!("x has the value {x}");
}

fn expect_value() {
    let guess = "42".parse().expect("Not a number");

    println!("guess: {guess}");
}
