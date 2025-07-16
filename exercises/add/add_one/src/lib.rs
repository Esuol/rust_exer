use rand::Rng;

pub fn add_one(x: i32) -> i32 {
    let mut rng = rand::rng();
    let y: i32 = rng.random_range(0..=100);
    println!("y: {}", y);
    x + 1 + y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(add_one(1), 2);
    }
}
