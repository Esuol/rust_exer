// 在模块级别定义
pub struct Guess {
    value: i32,
}

impl Guess {
    pub fn new(value: i32) -> Guess {
        if value < 1 || value > 100 {
            panic!("Guess value must be between 1 and 100, got {value}.");
        }

        Guess { value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

pub fn run_guess() {
    // 现在可以在函数中使用 Guess
    let guess = Guess::new(50);
    println!("Guess value: {}", guess.value());

    // 也可以测试边界情况
    // let invalid_guess = Guess::new(101); // 这会 panic
}
