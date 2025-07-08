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

    // 编译练习文件
    let compile_output = Command::new("rustc")
        .arg(exercise_path)
        .arg("-o")
        .arg("./temp_exercise")
        .output();

    match compile_output {
        Ok(output) => {
            if output.status.success() {
                // 编译成功，运行程序
                println!("编译成功，开始运行程序...");
                println!("----------------------------------------");

                let run_result = Command::new("./temp_exercise").spawn();

                match run_result {
                    Ok(mut child) => {
                        println!("child: {:?}", child.id());
                        // 等待程序执行完毕
                        let status = child.wait();
                        match status {
                            Ok(exit_status) => {
                                println!("----------------------------------------");
                                if exit_status.success() {
                                    println!("程序执行成功");
                                } else {
                                    println!("程序执行失败 (退出码: {})", exit_status);
                                }
                            }
                            Err(e) => {
                                println!("等待程序执行时出错: {}", e);
                            }
                        }
                    }
                    Err(e) => match e.kind() {
                        std::io::ErrorKind::NotFound => {
                            println!("错误: 找不到可执行文件 './temp_exercise'");
                        }
                        std::io::ErrorKind::PermissionDenied => {
                            println!("错误: 没有权限运行 './temp_exercise'");
                        }
                        _ => {
                            println!("运行错误: {}", e);
                        }
                    },
                }

                // 清理临时文件
                let _ = fs::remove_file("./temp_exercise");
            } else {
                // 编译失败
                println!("编译失败:");
                if !output.stdout.is_empty() {
                    println!("标准输出:");
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    println!("错误输出:");
                    println!("{}", String::from_utf8_lossy(&output.stderr));
                }
            }
        }
        Err(e) => println!("编译错误: {}", e),
    }
}
