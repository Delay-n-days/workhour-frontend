#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![recursion_limit = "256"]
//! 工时填报系统 - Tauri 主入口

mod commands;
mod eworkhour;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::validate_eteamsid,
            commands::get_projects,
            commands::get_work_types,
            commands::get_time_slots,
            commands::submit_workhour,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
