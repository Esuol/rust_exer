pub mod enums;

fn main() {
    run_struct();

    run_enum();

    enums::run_enum();

    enums::run_plus_one();
}

fn area(width: u32, height: u32) -> u32 {
    width * height
}

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

fn area1(rectangle: &Rectangle) -> u32 {
    rectangle.width * rectangle.height
}

#[derive(Debug)]
struct Rectangle2 {
    width: u32,
    height: u32,
}

impl Rectangle2 {
    fn from(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn can_hold(&self, other: &Rectangle2) -> bool {
        self.width > other.width && self.height > other.height
    }
}

#[derive(Debug)]
struct Rectangle3<'a> {
    width: u32,
    height: u32,
    name: &'a str,
}

impl<'a> Rectangle3<'a> {
    fn from(width: u32, height: u32, name: &'a str) -> Self {
        Self {
            width,
            height,
            name,
        }
    }

    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn can_hold(&self, other: &Rectangle3) -> bool {
        self.width > other.width && self.height > other.height
    }
}

fn run_struct() {
    let width1 = 30;
    let height1 = 50;

    println!(
        "The area of the rectangle is {} square pixels.",
        area(width1, height1)
    );

    // -----
    let rect1 = Rectangle {
        width: 30,
        height: 50,
    };

    println!("rect1 is {:?}", rect1);
    println!("rect1 is {:#?}", rect1);

    println!(
        "The area1 of the rectangle is {} square pixels.",
        area1(&rect1)
    );

    // dbg!
    let scale = 2;
    let rect2 = Rectangle {
        width: dbg!(30 * scale),
        height: 50,
    };

    dbg!(&rect2);

    // Rectangle2
    let rect3 = Rectangle2 {
        width: 30,
        height: 50,
    };

    let rect4 = Rectangle2 {
        width: 10,
        height: 40,
    };

    let rect5 = Rectangle2::from(20, 40);

    dbg!(&rect5);

    println!(
        "The area3 of the rectangle is {} square pixels.",
        rect3.area()
    );

    println!("Can rect3 hold rect4? {}", rect3.can_hold(&rect4));

    let rect6 = Rectangle3 {
        width: 30,
        height: 50,
        name: "rect6",
    };

    let rect7 = Rectangle3::from(20, 40, "rect7");

    let react6_area = rect6.area();

    println!(
        "The area6 of the rectangle is {} square pixels.",
        react6_area
    );

    println!("rect6 name is {}", rect6.name);

    println!("Can rect6 hold rect7? {}", rect6.can_hold(&rect7));

    dbg!(&rect6);
}

// ====== enum
enum IpAddr {
    V4(String),
    V6(String),
}

enum Fruit<'a> {
    Apple(&'a str),
    Orange(&'a str),
}

fn run_enum() {
    let home = IpAddr::V4(String::from("127.0.0.1"));

    let loopback = IpAddr::V6(String::from("::1"));

    let home_str = match home {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(ip) => ip,
    };

    let loopback_str = match loopback {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(ip) => ip,
    };

    println!("home_str: {home_str}");
    println!("loopback_str: {loopback_str}");

    let apple = Fruit::Apple("apple");
    let orange = Fruit::Orange("orange");

    let apple = match apple {
        Fruit::Apple(fruit) => fruit,
        Fruit::Orange(fruit) => fruit,
    };

    let orange = match orange {
        Fruit::Apple(fruit) => fruit,
        Fruit::Orange(fruit) => fruit,
    };

    println!("apple is: {apple}");
    println!("orange is: {orange}");

    println!("fruit is: {apple}");
}
