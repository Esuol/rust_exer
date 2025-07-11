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

impl UsState {
    fn existed_in(&self, year: u32) -> bool {
        match self {
            UsState::Alabama => year >= 1861,
            UsState::Alaska => year >= 1959,
            UsState::Arizona => year >= 1912,
            UsState::Arkansas => year >= 1836,
            UsState::California => year >= 1850,
        }
    }
}

fn describe_state_quarter(coin: Coin) -> Option<String> {
    let state = if let Coin::Quarter(state) = coin {
        state
    } else {
        return None;
    };
    if state.existed_in(1861) {
        Some(format!("{state:?} is pretty old, for America!"))
    } else {
        Some(format!("{state:?} is relatively new."))
    }
}

pub fn run_enum() {
    let value = value_in_cents(Coin::Quarter(UsState::Alabama));

    let value2 = value_in_cents(Coin::Penny);

    println!("The value of the coin is {} cents.", value);

    println!("The value2 of the coin is {} cents.", value2);

    let state_quarter = describe_state_quarter(Coin::Quarter(UsState::Alabama));

    println!("State quarter is {:?}", state_quarter);
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
