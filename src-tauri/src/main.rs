#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![recursion_limit = "256"]
//! 工时填报系统 - Tauri 主入口

mod commands;
mod eworkhour;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 用户尝试开第二个实例时，把已有窗口拉到前台
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        // 添加日志插件 - 日志保存在应用目录下的 logs 文件夹
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)  // 日志级别: Error, Warn, Info, Debug, Trace
                .max_file_size(5_000_000)       // 单个日志文件最大 5MB
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)  // 保留所有日志文件
                .timezone_strategy(tauri_plugin_log::TimestampStrategy::UseLocal)  // 使用本地时间
                .build()
        )
        .invoke_handler(tauri::generate_handler![
            commands::validate_eteamsid,
            commands::get_projects,
            commands::get_work_types,
            commands::get_time_slots,
            commands::submit_workhour,
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            // 打印日志目录路径
            let log_dir = app.path().app_log_dir().unwrap_or_default();
            log::info!("应用启动，日志目录: {:?}", log_dir);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
