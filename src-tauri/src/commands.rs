//! Tauri 命令层
//! 暴露给前端 invoke() 调用的函数，对应工时系统 API

use crate::eworkhour::{EworkhourClient, WorkHourEntry};

/// 验证 eteamsid
#[tauri::command]
pub async fn validate_eteamsid(eteamsid: String) -> Result<serde_json::Value, String> {
    eprintln!("[CMD] validate_eteamsid: eteamsid='{}'", eteamsid);

    let client = EworkhourClient::new();
    let result = client.validate_eteamsid(&eteamsid)
        .await
        .map_err(|e| {
            let msg = format!("验证失败: {}", e);
            eprintln!("[CMD] validate_eteamsid ERROR: {}", msg);
            msg
        })?;

    eprintln!("[CMD] validate_eteamsid: valid={}, employee_id={:?}, employee_name={:?}",
        result.valid, result.employee_id, result.employee_name);

    Ok(serde_json::json!({
        "valid": result.valid,
        "employee_id": result.employee_id,
        "employee_name": result.employee_name,
        "message": result.message,
    }))
}

/// 获取项目列表（静态）
#[tauri::command]
pub async fn get_projects() -> Result<serde_json::Value, String> {
    eprintln!("[CMD] get_projects");

    let client = EworkhourClient::new();
    let result = client.get_projects()
        .await
        .map_err(|e| {
            let msg = format!("获取项目列表失败: {}", e);
            eprintln!("[CMD] get_projects ERROR: {}", msg);
            msg
        })?;

    eprintln!("[CMD] get_projects: 成功");
    Ok(result)
}

/// 获取工作类型列表（静态）
#[tauri::command]
pub async fn get_work_types() -> Result<serde_json::Value, String> {
    eprintln!("[CMD] get_work_types");

    let client = EworkhourClient::new();
    let result = client.get_work_types()
        .await
        .map_err(|e| {
            let msg = format!("获取工作类型失败: {}", e);
            eprintln!("[CMD] get_work_types ERROR: {}", msg);
            msg
        })?;

    eprintln!("[CMD] get_work_types: 成功");
    Ok(result)
}

/// 获取时间段列表
#[tauri::command]
pub async fn get_time_slots() -> Result<serde_json::Value, String> {
    eprintln!("[CMD] get_time_slots");

    let client = EworkhourClient::new();
    let result = client.get_time_slots()
        .await
        .map_err(|e| {
            let msg = format!("获取时间段失败: {}", e);
            eprintln!("[CMD] get_time_slots ERROR: {}", msg);
            msg
        })?;

    eprintln!("[CMD] get_time_slots: 成功");
    Ok(result)
}

/// 提交工时
#[tauri::command]
pub async fn submit_workhour(
    eteamsid: String,
    work_date: String,
    entries: Vec<WorkHourEntry>,
    employee_id: String,
    employee_name: String,
) -> Result<serde_json::Value, String> {
    eprintln!("[CMD] submit_workhour: eteamsid='{}', work_date='{}', entries={}, employee_id='{}', employee_name='{}'",
        eteamsid, work_date, entries.len(), employee_id, employee_name);

    let client = EworkhourClient::new();
    let result = client.submit_workhour(&eteamsid, &work_date, &entries, &employee_id, &employee_name)
        .await
        .map_err(|e| {
            let msg = format!("提交工时失败: {}", e);
            eprintln!("[CMD] submit_workhour ERROR: {}", msg);
            msg
        })?;

    eprintln!("[CMD] submit_workhour: success={}, message='{}'", result.success, result.message);

    Ok(serde_json::json!({
        "success": result.success,
        "message": result.message,
        "request_name": result.request_name,
        "result_message": result.result_message,
        "record_id": result.record_id,
    }))
}
