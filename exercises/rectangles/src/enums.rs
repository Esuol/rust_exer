#[derive(Debug)]
enum UsState {
    Alabama,
    Alaska,
    Arizona,
    Arkansas,
    California,
}

enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter(UsState),
}

fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter(state) => {
            println!("State quarter from {:?}!", state);
            25
        }
    }
}

pub fn run_enum() {
    let value = value_in_cents(Coin::Quarter(UsState::Alabama));

    let value2 = value_in_cents(Coin::Penny);

    println!("The value of the coin is {} cents.", value);

    println!("The value2 of the coin is {} cents.", value2);
}

pub fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1),
    }
}

pub fn run_plus_one() {
    let five = Some(5);
    let six = plus_one(five);
    let none = plus_one(None);

    println!("six is {:?}", six);
    println!("none is {:?}", none);
}
