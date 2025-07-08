fn main() {
    let x = 5;

    println!("x has the value {x}");

    loop_value()
}

fn loop_value() {
    let mut counter = 0;

    'counting_up: loop {
        println!("counter = {counter}");
        let mut remaining = 10;

        loop {
            println!("remaining = {remaining}");

            if remaining == 9 {
                break;
            }

            if counter == 2 {
                break 'counting_up;
            }

            remaining -= 1;
        }

        counter += 1;
    }

    println!("End count = {counter}");
}
