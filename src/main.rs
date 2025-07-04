use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("使用方法: cargo run <练习文件路径>");
        println!("例如: cargo run exercises/01_variables/variables1.rs");
        return;
    }

    let exercise_path = &args[1];

    // 检查文件是否存在
    if !fs::metadata(exercise_path).is_ok() {
        println!("错误: 文件 '{}' 不存在", exercise_path);
        return;
    }

    // 编译并运行练习
    let output = Command::new("rustc")
        .arg(exercise_path)
        .arg("-o")
        .arg("temp_exercise")
        .output();

    match output {
        Ok(_) => {
            // 运行编译后的程序
            let run_result = Command::new("./temp_exercise").output();
            match run_result {
                Ok(output) => {
                    println!("输出:");
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                    if !output.stderr.is_empty() {
                        println!("错误:");
                        println!("{}", String::from_utf8_lossy(&output.stderr));
                    }
                }
                Err(e) => println!("运行错误: {}", e),
            }

            // 清理临时文件
            let _ = fs::remove_file("temp_exercise");
        }
        Err(e) => println!("编译错误: {}", e),
    }
}
