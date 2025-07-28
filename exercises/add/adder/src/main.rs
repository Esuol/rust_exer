use guess::run_guess;
use pair::Pair;
use std::collections::HashMap;
use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;
use summary::{NewsArticle, SocialPost, Summary};

mod guess;
mod pair;
mod summary;

fn main() {
    let num = 10;
    println!("Hello, world! {num} plus one is {}!", add_one::add_one(num));
    run_str();
    run_hash_map();
    run_result();
    run_result_2();

    match run_result_3() {
        Ok(username) => println!("Final username: {}", username),
        Err(e) => println!("Error: {}", e),
    }

    run_guess();

    let number_list = vec![34, 50, 25, 100, 65];

    let result = largest(&number_list);
    println!("The largest number is {result}");

    let char_list = vec!['y', 'm', 'a', 'q'];

    let result = largest(&char_list);
    println!("The largest char is {result}");

    run_summary();

    run_longest();
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

fn run_hash_map() {
    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);
    scores.entry(String::from("Blue")).or_insert(25);
    scores.entry(String::from("Red")).or_insert(50);

    let team_name = String::from("Blue");
    let score = scores.get(&team_name).copied().unwrap_or(0);
    println!("score is {score}");

    for (key, value) in &scores {
        println!("{key}: {value}");
    }

    let text = "hello world wonderful world";

    let mut word_map = HashMap::new();

    for word in text.split_whitespace() {
        let count: &mut i32 = word_map.entry(word).or_insert(0);
        *count += 1;
    }

    println!("{:?}", word_map);
}

fn run_result() {
    let greeting_file_result = File::open("./hello.txt");

    match greeting_file_result {
        Ok(mut file) => {
            println!("File opened successfully!");
            // 这里可以处理文件
            let mut file_content = String::new();
            file.read_to_string(&mut file_content).unwrap();
            println!("File content: {file_content}");
        }
        Err(e) => {
            println!("Could not open file: {e}");
            println!("Creating a new file...");

            // 创建文件
            match File::create("./hello.txt") {
                Ok(_) => println!("File created successfully!"),
                Err(e) => println!("Could not create file: {e}"),
            }
        }
    };
}

fn run_result_2() {
    let greeting_file_result = File::open("./hello2.txt");

    let greeting_file = match greeting_file_result {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create("./hello2.txt") {
                Ok(file) => file,
                Err(e) => panic!("Problem creating the file: {e}"),
            },
            _ => {
                panic!("Problem opening the file: {error:?}");
            }
        },
    };
}

fn run_result_3() -> Result<String, std::io::Error> {
    let greeting_file_result = File::open("./hell12.txt");

    let mut username_file = match greeting_file_result {
        Ok(file) => file,
        Err(_) => {
            println!("File not found, using default");
            return Ok(String::from("default"));
        }
    };

    let mut username = String::new();

    let c = match username_file.read_to_string(&mut username) {
        Ok(_) => Ok(username),
        Err(e) => Err(e),
    }
    .unwrap_or_else(|_| String::from("default"));

    println!("username: {c}");

    Ok(c)
}

fn run_result_4() {
    let greeting_file_result = File::open("./hello2.txt");

    let greeting_file = match greeting_file_result {
        Ok(file) => file,
        Err(e) => panic!("Problem opening the file: {e}"),
    };
}

pub fn largest<T: std::cmp::PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];

    for item in list {
        if item > largest {
            largest = item;
        }
    }

    largest
}

fn run_summary() {
    let post = SocialPost {
        username: String::from("horse_ebooks"),
        content: String::from("of course, as you probably already know, people"),
        reply: false,
        retweet: false,
    };

    println!("1 new post: {}", post.summarize());

    let article = NewsArticle {
        headline: String::from("Penguins win the Stanley Cup Championship!"),
        location: String::from("Pittsburgh, PA, USA"),
        author: String::from("Iceburgh"),
        content: String::from("The Pittsburgh Penguins once again are the best"),
    };

    println!("New article available! {}", article.summarize());
}

fn run_pair() {
    let pair = Pair::new(1, 2);
    pair.cmp_display();
}

fn run_result_5() {
    let f = File::open("hello.txt");
    let f = match f {
        Ok(file) => file,
        Err(e) => {
            panic!("Problem opening the file: {e}");
        }
    };

    let f = File::open("hello.txt").unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound {
            File::create("hello.txt").unwrap_or_else(|error| {
                panic!("Problem creating the file: {error}");
            })
        } else {
            panic!("Problem opening the file: {error}");
        }
    });
}

fn run_result_6() {
    let f = File::open("hello.txt").unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound {
            File::create("hello.txt").unwrap_or_else(|error| {
                panic!("Problem creating the file: {error}");
            })
        } else {
            panic!("Problem opening the file: {error}");
        }
    });
}

fn run_result_7() {
    let f = File::open("hello.txt").unwrap();
    let f = File::open("hello.txt").expect("Failed to open hello.txt");
}

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

fn run_longest() {
    let string1 = String::from("abcd");
    let string2 = "xyz";

    let result = longest(string1.as_str(), string2);
    println!("The longest string is {result}");
}
