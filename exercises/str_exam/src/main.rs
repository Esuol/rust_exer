// Day 1-3 实战练习: 变量与数据类型
// 在你的项目中创建 src/main.rs，复制这些代码逐个运行测试

fn main() {
    println!("=== Rust学习 Day 1-3 ===");

    // 练习1: 变量声明与可变性
    variable_mutability_demo();

    // 练习2: 数据类型探索
    data_types_demo();

    // 练习3: 字符串处理 (重要!)
    string_handling_demo();

    // 练习4: 元组和数组
    collections_demo();

    // 实战项目: 用户信息处理器
    user_info_processor();
}

fn variable_mutability_demo() {
    println!("\n--- 变量可变性演示 ---");

    // 不可变变量
    let x = 5;
    println!("不可变变量 x: {}", x);

    // 可变变量
    let mut y = 10;
    println!("可变变量 y (初始): {}", y);
    y = 15;
    println!("可变变量 y (修改后): {}", y);

    // 变量遮蔽 (shadowing) - Rust特色功能
    let spaces = "   ";
    println!("spaces 作为字符串: '{}'", spaces);
    let spaces = spaces.len(); // 类型从 &str 变为 usize
    println!("spaces 作为数字: {}", spaces);

    // 常量
    const MAX_USERS: u32 = 1000;
    println!("系统最大用户数: {}", MAX_USERS);
}

fn data_types_demo() {
    println!("\n--- 数据类型演示 ---");

    // 整数类型
    let small_num: i8 = 127;
    let big_num: i64 = 1_000_000;
    let hex = 0xff;
    let octal = 0o77;
    let binary = 0b1111_0000;

    println!(
        "各种整数: {}, {}, {}, {}, {}",
        small_num, big_num, hex, octal, binary
    );

    // 浮点数
    let pi: f32 = 3.14159;
    let e = 2.718281828; // 默认 f64
    println!("浮点数: {}, {}", pi, e);

    // 布尔值和字符
    let is_rust_awesome = true;
    let emoji = '🦀'; // Rust的吉祥物!
    println!("布尔值: {}, Rust吉祥物: {}", is_rust_awesome, emoji);

    // 数学运算
    let sum = 5 + 10;
    let difference = 95.5 - 4.3;
    let product = 4 * 30;
    let quotient = 56.7 / 32.2;
    let remainder = 43 % 5;

    println!(
        "运算结果: {}, {}, {}, {}, {}",
        sum, difference, product, quotient, remainder
    );
}

fn string_handling_demo() {
    println!("\n--- 字符串处理演示 (重要概念!) ---");

    // 字符串字面量 (&str) - 存储在程序二进制中
    let greeting = "Hello";
    println!("字符串字面量: {}", greeting);

    // String 类型 - 堆分配，可变
    let mut message = String::from("Hello");
    message.push_str(", Rust!");
    println!("String 类型: {}", message);

    // 字符串方法
    let len = message.len();
    let contains_rust = message.contains("Rust");
    let uppercase = message.to_uppercase();

    println!(
        "长度: {}, 包含Rust: {}, 大写: {}",
        len, contains_rust, uppercase
    );

    // 字符串切片
    let slice = &message[0..5];
    println!("字符串切片 [0..5]: {}", slice);
}

fn collections_demo() {
    println!("\n--- 集合类型演示 ---");

    // 元组 - 不同类型的组合
    let person_info: (&str, i32, f64, bool) = ("Alice", 30, 175.5, true);
    let (name, age, height, is_developer) = person_info; // 解构

    println!(
        "姓名: {}, 年龄: {}, 身高: {}cm, 是开发者: {}",
        name, age, height, is_developer
    );

    // 通过索引访问
    println!("通过索引访问年龄: {}", person_info.1);

    // 数组 - 固定大小，相同类型
    let fibonacci = [1, 1, 2, 3, 5, 8, 13];
    let zeros = [0; 5]; // [0, 0, 0, 0, 0]

    println!("斐波那契数列: {:?}", fibonacci);
    println!("零数组: {:?}", zeros);

    // 数组访问
    println!("第一个斐波那契数: {}", fibonacci[0]);
    println!("数组长度: {}", fibonacci.len());
}

fn user_info_processor() {
    println!("\n--- 实战项目: 用户信息处理器 ---");

    // 模拟用户数据
    let users = [
        ("张三", 25, "backend", 8500.0),
        ("李四", 28, "frontend", 9200.0),
        ("王五", 32, "fullstack", 12000.0),
    ];

    println!("=== 公司员工信息 ===");

    let mut total_salary = 0.0;
    let mut senior_count = 0;

    for (i, user) in users.iter().enumerate() {
        let (name, age, position, salary) = user;

        // 判断资深程度
        let experience_level = if *age >= 30 {
            senior_count += 1;
            "资深"
        } else {
            "初中级"
        };

        total_salary += salary;

        println!(
            "{}. 姓名: {}, 年龄: {}, 职位: {}, 薪资: ¥{:.0}, 级别: {}",
            i + 1,
            name,
            age,
            position,
            salary,
            experience_level
        );
    }

    let avg_salary = total_salary / users.len() as f64;

    println!("\n=== 统计信息 ===");
    println!("总员工数: {}", users.len());
    println!("资深员工数: {}", senior_count);
    println!("平均薪资: ¥{:.0}", avg_salary);
    println!("总薪资成本: ¥{:.0}", total_salary);

    // 寻找最高薪资
    let highest_salary = users
        .iter()
        .map(|(_, _, _, salary)| salary)
        .fold(0.0, |acc, &salary| if salary > acc { salary } else { acc });

    println!("最高薪资: ¥{:.0}", highest_salary);
}

// 额外挑战: 类型转换练习
#[allow(dead_code)]
fn type_conversion_challenge() {
    println!("\n--- 类型转换挑战 ---");

    // 数字转换
    let integer = 42;
    let float = integer as f64;
    let back_to_int = float as i32;

    println!(
        "整数: {} -> 浮点: {} -> 整数: {}",
        integer, float, back_to_int
    );

    // 字符串转数字
    let string_number = "123";
    let parsed_number: i32 = string_number.parse().expect("解析失败");
    println!("字符串 '{}' 转数字: {}", string_number, parsed_number);

    // 数字转字符串
    let number = 456;
    let string_from_number = number.to_string();
    println!("数字 {} 转字符串: '{}'", number, string_from_number);
}
