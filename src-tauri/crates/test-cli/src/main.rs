//! 工时系统 API 测试工具
//! 用法: cargo run -p test-cli -- <eteamsid> [command]

use eworkhour::EworkhourClient;
use std::env;

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("工时系统 API 测试工具");
        println!();
        println!("用法:");
        println!("  cargo run -p test-cli -- <eteamsid> [command]");
        println!();
        println!("命令:");
        println!("  validate     - 验证 eteamsid");
        println!("  projects     - 获取项目列表");
        println!("  worktypes    - 获取工作类型列表");
        println!("  timeslots    - 获取时间段列表");
        println!("  all          - 执行所有命令");
        println!();
        println!("示例:");
        println!("  cargo run -p test-cli -- THIRDSSO_xxx validate");
        println!("  cargo run -p test-cli -- THIRDSSO_xxx projects");
        println!("  cargo run -p test-cli -- THIRDSSO_xxx all");
        return;
    }

    let eteamsid = &args[1];
    let command = if args.len() > 2 { args[2].clone() } else { "all".to_string() };

    println!("========================================");
    println!("工时系统 API 测试");
    println!("eteamsid: {}", eteamsid);
    println!("command: {}", command);
    println!("========================================");
    println!();

    let client = EworkhourClient::new();

    match command.as_str() {
        "validate" => {
            println!(">>> 验证 eteamsid...");
            match client.validate_eteamsid(eteamsid).await {
                Ok(result) => {
                    println!("结果: {:?}", result);
                }
                Err(e) => {
                    eprintln!("错误: {}", e);
                }
            }
        }
        "projects" => {
            println!(">>> 获取项目列表...");
            match client.get_projects(eteamsid).await {
                Ok(result) => {
                    let projects = result.get("projects").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    println!("共 {} 个项目:", projects.len());
                    for (i, p) in projects.iter().take(10).enumerate() {
                        let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        println!("  {}. {} ({})", i + 1, name, id);
                    }
                    if projects.len() > 10 {
                        println!("  ... 还有 {} 个项目", projects.len() - 10);
                    }
                }
                Err(e) => {
                    eprintln!("错误: {}", e);
                }
            }
        }
        "worktypes" => {
            println!(">>> 获取工作类型列表...");
            match client.get_work_types(eteamsid).await {
                Ok(result) => {
                    let work_types = result.get("work_types").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    println!("共 {} 种工作类型:", work_types.len());
                    for (i, wt) in work_types.iter().enumerate() {
                        let id = wt.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = wt.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        println!("  {}. {} ({})", i + 1, name, id);
                    }
                }
                Err(e) => {
                    eprintln!("错误: {}", e);
                }
            }
        }
        "timeslots" => {
            println!(">>> 获取时间段列表...");
            match client.get_time_slots().await {
                Ok(result) => {
                    let time_slots = result.get("time_slots").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    println!("时间段:");
                    for ts in time_slots {
                        let id = ts.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = ts.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        println!("  {} - {}", id, name);
                    }
                }
                Err(e) => {
                    eprintln!("错误: {}", e);
                }
            }
        }
        "all" => {
            println!(">>> 执行所有命令...\n");

            // 验证
            println!("1. 验证 eteamsid:");
            match client.validate_eteamsid(eteamsid).await {
                Ok(result) => {
                    println!("   valid: {}", result.valid);
                    println!("   employee_id: {:?}", result.employee_id);
                    println!("   employee_name: {:?}", result.employee_name);
                }
                Err(e) => {
                    eprintln!("   错误: {}", e);
                }
            }
            println!();

            // 项目列表
            println!("2. 获取项目列表:");
            match client.get_projects(eteamsid).await {
                Ok(result) => {
                    let projects = result.get("projects").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    println!("   共 {} 个项目", projects.len());
                    for (i, p) in projects.iter().take(5).enumerate() {
                        let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        println!("   {}. {}", i + 1, name);
                    }
                }
                Err(e) => {
                    eprintln!("   错误: {}", e);
                }
            }
            println!();

            // 工作类型
            println!("3. 获取工作类型列表:");
            match client.get_work_types(eteamsid).await {
                Ok(result) => {
                    let work_types = result.get("work_types").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    println!("   共 {} 种工作类型", work_types.len());
                    for (i, wt) in work_types.iter().enumerate() {
                        let name = wt.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        println!("   {}. {}", i + 1, name);
                    }
                }
                Err(e) => {
                    eprintln!("   错误: {}", e);
                }
            }
            println!();

            // 时间段
            println!("4. 获取时间段列表:");
            match client.get_time_slots().await {
                Ok(result) => {
                    let time_slots = result.get("time_slots").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                    for ts in time_slots {
                        let id = ts.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = ts.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        println!("   {} - {}", id, name);
                    }
                }
                Err(e) => {
                    eprintln!("   错误: {}", e);
                }
            }
        }
        _ => {
            eprintln!("未知命令: {}", command);
            eprintln!("可用命令: validate, projects, worktypes, timeslots, all");
        }
    }

    println!();
    println!("完成!");
}
