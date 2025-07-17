fn main() {
    let num = 10;
    println!("Hello, world! {num} plus one is {}!", add_one::add_one(num));
    run_str();
}

fn run_str() {
    let data = "initial contents";
    let s = data.to_string();
    println!("data 的类型: {}", std::any::type_name_of_val(&data)); // &str
    println!("s 的类型: {}", std::any::type_name_of_val(&s));
    println!("data 的地址: {:p}", data); // 指向字符串字面量
    println!("s 的地址: {:p}", &s); // 指向堆内存

    let mut s1 = String::from("foo");
    let s2 = "bar";
    s1.push_str(s2);
    println!("s2 is {s2}");

    let s1 = String::from("tic");
    let s2 = String::from("tac");
    let s3 = String::from("toe");

    let s = format!("{s1}-{s2}-{s3}");
    println!("s is {s}");
    println!("s 的类型: {}", std::any::type_name_of_val(&s));
}
