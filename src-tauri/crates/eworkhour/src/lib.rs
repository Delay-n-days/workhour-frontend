#![recursion_limit = "256"]
//! 工时系统 API 客户端库

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use log::{info, error, debug};

// 流程 / 表单固定 ID
const WORKFLOW_ID: &str = "1215170926745124864";
const NODE_ID: &str = "1215170926745124866";
const FORM_ID: &str = "1215162010367852547";
const FORM_LAYOUT_ID: &str = "1215162010367852618";
const CUS_MENU_ID: &str = "1192823941250891835";
const FIELD_WORK_TYPE: &str = "1249973841324212226";

/// 工时系统 API 客户端
pub struct EworkhourClient {
    client: Client,
    base_url: String,
}

/// 验证响应
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateResponse {
    pub valid: bool,
    pub employee_id: Option<String>,
    pub employee_name: Option<String>,
    pub message: Option<String>,
}

/// 工时条目
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkHourEntry { 
    pub time_slot_id: String,
    pub project_id: String,
    pub project_name: String,
    pub hours: String,
    pub work_type_id: String,
    pub work_type_name: String,
}

/// 提交响应
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitResponse {
    pub success: bool,
    pub message: String,
    pub request_name: Option<String>,
    pub result_message: Option<String>,
    pub record_id: Option<String>,
}

impl EworkhourClient { 
    /// 创建新的客户端实例
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(30))
                .gzip(true)
                .build()
                .unwrap(),
            base_url: "http://202.105.113.101:20600".to_string(),
        }
    }

    /// 发送 HTTP 请求
    async fn make_request(
        &self,
        method: &str,
        path: &str,
        body: &Value,
        eteamsid: &str,
    ) -> Result<Value, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        debug!("[HTTP] {} {}", method, url);

        let referer = format!("{}/", self.base_url);
        let mut headers = HashMap::new();
        headers.insert("Accept", "application/json, text/plain, */*");
        headers.insert("Accept-Encoding", "gzip, deflate");
        headers.insert("Accept-Language", "zh-CN,zh;q=0.9");
        headers.insert("Content-Type", "application/json;charset=UTF-8");
        headers.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36 Edg/149.0.0.0");
        headers.insert("eteamsid", eteamsid);
        headers.insert("langType", "zh_CN");
        headers.insert("timeZoneOffset", "-480");
        headers.insert("Origin", &self.base_url);
        headers.insert("Referer", &referer);

        let mut request = if method_upper(method) {
            self.client.post(&url)
        } else {
            self.client.get(&url)
        };

        for (key, value) in &headers {
            request = request.header(*key, *value);
        }

        let response = if method_upper(method) {
            request.json(body).send().await?
        } else {
            request.send().await?
        };

        let status = response.status();
        debug!("[HTTP] 响应状态: {}", status);

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            error!("[HTTP] 请求失败: status={}, body={}", status, &text[..text.len().min(500)]);
            return Ok(json!({"status": false, "msg": format!("HTTP {}: {}", status, &text[..text.len().min(200)])}));
        }

        let text = response.text().await.unwrap_or_default();
        debug!("[HTTP] 响应内容: {}", &text[..text.len().min(500)]);

        match serde_json::from_str::<Value>(&text) {
            Ok(v) => Ok(v),
            Err(e) => {
                error!("[HTTP] JSON 解析失败: {}, 原始内容: {}", e, &text[..text.len().min(200)]);
                Ok(json!({"status": false, "msg": format!("JSON 解析失败: {}", e)}))
            }
        }
    }

    /// 验证 eteamsid
    pub async fn validate_eteamsid(&self, eteamsid: &str) -> Result<ValidateResponse, reqwest::Error> {
        info!("[API] 开始验证 eteamsid...");
        debug!("[API] eteamsid='{}'", eteamsid);

        let body = json!({
            "cusMenuId": "1192823941250891835",
            "urlPageTitle": "5a6e6ZmF5bel5pe2",
            "isCreate": 1,
            "workflowId": "1215170926745124864",
            "id": "1215170926745124864",
            "fieldAssignKeys": "",
            "jumplinkParamKey": "SubmitApplication",
        });

        let data = self.make_request("POST", "/api/workflow/core/flowPage/loadBaseParam", &body, eteamsid).await?;
        debug!("[API] loadBaseParam 响应: {:?}", data);

        if data.get("status") == Some(&Value::Bool(false)) {
            let error_msg = data.get("msg").and_then(|v| v.as_str()).unwrap_or("eteamsid 无效或已过期");
            error!("[API] eteamsid 验证失败: {}", error_msg);
            return Ok(ValidateResponse {
                valid: false,
                employee_id: None,
                employee_name: None,
                message: Some(error_msg.to_string()), 
            });
        }

        let employee_id = data.get("data")
            .and_then(|d| d.get("userInfo"))
            .and_then(|u| u.get("employeeId"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let employee_name = data.get("data")
            .and_then(|d| d.get("userInfo"))
            .and_then(|u| u.get("employeeName"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        info!("[API] eteamsid 验证成功: employee_id={:?}, employee_name={:?}", employee_id, employee_name);

        Ok(ValidateResponse {
            valid: true,
            employee_id,
            employee_name,
            message: Some("eteamsid 验证成功".to_string()),
        })
    }

    /// 获取项目列表（动态）
    pub async fn get_projects(&self, eteamsid: &str) -> Result<Value, reqwest::Error> {
        info!("[API] 开始获取项目列表...");
        debug!("[API] 请求参数: eteamsid='{}'", eteamsid);

        let form_param = json!({
            "formId": FORM_ID,
            "layoutId": FORM_LAYOUT_ID,
            "fieldId": "1215162323950796801",
            "filterItems": [],
            "module": "ebuildercard",
            "dataDetails": []
        });
        let form_param_str = serde_json::to_string(&form_param).unwrap_or_default();

        let body = json!({
            "browserMultiple": false,
            "ebBrowserParams": {
                "openScroll": true,
                "variableDataKeyMap": {},
                "businessId": WORKFLOW_ID,
                "browserModule": "ebuilder",
                "formParam": form_param_str.clone(),
                "controlId": "1215162323950796801",
                "variableMap": {},
                "browserFullRoute": "ebuilder/form",
                "customParam": {
                    "permissionType": 1,
                    "requestId": "",
                    "workflowId": WORKFLOW_ID,
                    "nodeId": NODE_ID,
                    "authSignatureStr": "",
                    "authStr": "",
                    "readOnly": false,
                    "identityId": "",
                    "identityType": 0,
                    "isTest": false,
                    "create": true,
                    "mobile": false,
                    "formId": FORM_ID,
                    "source": "view",
                    "printConfig": {},
                    "bridgeService": "ebuilder",
                    "isAgent": false,
                    "beAgentId": "0",
                    "hideSysFieldArea": true,
                    "viewFormNodeId": NODE_ID,
                    "turnTodoUser": [],
                    "sysFieldNewMode": true,
                },
                "cusMenuId": CUS_MENU_ID,
                "browserType": "ebuilder",
                "browserMultiple": false,
                "browserTabConfigList": [],
                "compId": "fdc5b9be6c3b4fab8d2c113323ba7654",
                "isMobile": false,
                "browserRoute": "ebuilder/form",
                "browserMethod": "prop",
                "urlPageTitle": "5a6e6ZmF5bel5pe2",
            },
            "pageSize": 100,
            "current": 1,
            "formParam": form_param_str,
            "openScroll": true,
            "controlId": "1215162323950796801",
            "businessId": WORKFLOW_ID,
            "variableMap": {},
            "customParam": {
                "permissionType": 1,
                "requestId": "",
                "workflowId": WORKFLOW_ID,
                "nodeId": NODE_ID,
                "authSignatureStr": "",
                "authStr": "",
                "readOnly": false,
                "identityId": "",
                "identityType": 0,
                "isTest": false,
                "create": true,
                "mobile": false,
                "formId": FORM_ID,
                "source": "view",
                "printConfig": {},
                "bridgeService": "ebuilder",
                "isAgent": false,
                "beAgentId": "0",
                "hideSysFieldArea": true,
                "viewFormNodeId": NODE_ID,
                "turnTodoUser": [],
                "sysFieldNewMode": true,
            },
        });

        let data = self.make_request("POST", "/api/ebuilder/form/common/browser/data/ebuilder", &body, eteamsid).await?;
        debug!("[API] get_projects 响应: {:?}", &data.to_string()[..data.to_string().len().min(500)]);

        if data.get("status") == Some(&Value::Bool(false)) {
            let error_msg = data.get("msg").and_then(|v| v.as_str()).unwrap_or("未知错误");
            error!("[API] 获取项目列表失败: {}", error_msg);
            return Ok(json!({"projects": [], "source": "static", "error": error_msg}));
        }

        let projects = data.get("data")
            .and_then(|d| d.get("data"))
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter().map(|p| {
                    json!({
                        "id": p.get("id"),
                        "name": p.get("content").or(p.get("name")).and_then(|v| v.as_str()).unwrap_or("")
                    })
                }).collect::<Vec<_>>()
            })
            .unwrap_or_default();

        info!("[API] 获取项目列表成功: 共 {} 个项目", projects.len());
        Ok(json!({"projects": projects, "source": "server"}))
    }

    /// 获取工作类型列表（动态）
    pub async fn get_work_types(&self, eteamsid: &str) -> Result<Value, reqwest::Error> {
        info!("[API] 开始获取工作类型列表...");
        debug!("[API] 请求参数: eteamsid='{}'", eteamsid);

        let form_param = json!({
            "formId": FORM_ID,
            "layoutId": FORM_LAYOUT_ID,
            "fieldId": FIELD_WORK_TYPE,
            "filterItems": [],
            "module": "ebuildercard",
            "dataDetails": []
        });
        let form_param_str = serde_json::to_string(&form_param).unwrap_or_default();

        let body = json!({
            "browserMultiple": true,
            "ebBrowserParams": {
                "openScroll": true,
                "variableDataKeyMap": {},
                "businessId": WORKFLOW_ID,
                "browserModule": "ebuilder",
                "formParam": form_param_str.clone(),
                "controlId": FIELD_WORK_TYPE,
                "variableMap": {},
                "browserFullRoute": "ebuilder/form",
                "customParam": {
                    "permissionType": 1,
                    "requestId": "",
                    "workflowId": WORKFLOW_ID,
                    "nodeId": NODE_ID,
                    "authSignatureStr": "",
                    "authStr": "",
                    "readOnly": false,
                    "identityId": "",
                    "identityType": 0,
                    "isTest": false,
                    "create": true,
                    "mobile": false,
                    "formId": FORM_ID,
                    "source": "view",
                    "printConfig": {},
                    "bridgeService": "ebuilder",
                    "isAgent": false,
                    "beAgentId": "0",
                    "hideSysFieldArea": true,
                    "viewFormNodeId": NODE_ID,
                    "turnTodoUser": [],
                    "sysFieldNewMode": true,
                },
                "cusMenuId": CUS_MENU_ID,
                "browserType": "ebuilder",
                "browserMultiple": true,
                "browserTabConfigList": [],
                "compId": "fdc5b9be6c3b4fab8d2c113323ba7654",
                "isMobile": false,
                "browserRoute": "ebuilder/form",
                "browserMethod": "prop",
                "urlPageTitle": "5a6e6ZmF5bel5pe2",
            },
            "pageSize": 50,
            "current": 1,
            "formParam": form_param_str,
            "openScroll": true,
            "controlId": FIELD_WORK_TYPE,
            "businessId": WORKFLOW_ID,
            "variableMap": {},
            "customParam": {
                "permissionType": 1,
                "requestId": "",
                "workflowId": WORKFLOW_ID,
                "nodeId": NODE_ID,
                "authSignatureStr": "",
                "authStr": "",
                "readOnly": false,
                "identityId": "",
                "identityType": 0,
                "isTest": false,
                "create": true,
                "mobile": false,
                "formId": FORM_ID,
                "source": "view",
                "printConfig": {},
                "bridgeService": "ebuilder",
                "isAgent": false,
                "beAgentId": "0",
                "hideSysFieldArea": true,
                "viewFormNodeId": NODE_ID,
                "turnTodoUser": [],
                "sysFieldNewMode": true,
            },
        });

        let data = self.make_request("POST", "/api/ebuilder/form/common/browser/data/ebuilder", &body, eteamsid).await?;
        debug!("[API] get_work_types 响应: {:?}", &data.to_string()[..data.to_string().len().min(500)]);

        if data.get("status") == Some(&Value::Bool(false)) {
            let error_msg = data.get("msg").and_then(|v| v.as_str()).unwrap_or("未知错误");
            error!("[API] 获取工作类型列表失败: {}", error_msg);
            return Ok(json!({"work_types": [], "source": "static", "error": error_msg}));
        }

        let work_types = data.get("data")
            .and_then(|d| d.get("data"))
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter().map(|wt| {
                    json!({
                        "id": wt.get("id"),
                        "name": wt.get("content")
                    })
                }).collect::<Vec<_>>()
            })
            .unwrap_or_default();

        info!("[API] 获取工作类型列表成功: 共 {} 种类型", work_types.len());
        Ok(json!({"work_types": work_types, "source": "server"}))
    }

    /// 获取时间段列表
    pub async fn get_time_slots(&self) -> Result<Value, reqwest::Error> {
        Ok(json!({
            "time_slots": [
                {"id": "0", "name": "10:30"},
                {"id": "1", "name": "13:40"},
                {"id": "2", "name": "15:40"},
                {"id": "3", "name": "17:40"},
            ]
        }))
    }

    /// 提交工时
    pub async fn submit_workhour(
        &self,
        eteamsid: &str,
        work_date: &str,
        entries: &[WorkHourEntry],
        employee_id: &str,
        employee_name: &str,
    ) -> Result<SubmitResponse, reqwest::Error> {
        info!("[API] 开始提交工时...");
        info!("[API] 工作日期: {}, 条目数: {}, 员工: {}", work_date, entries.len(), employee_name);
        debug!("[API] eteamsid='{}', employee_id='{}'", eteamsid, employee_id);

        let workflow_id = "1215170926745124864";
        let node_id = "1215170926745124866";
        let form_id = "1215162010367852547";
        let form_layout_id = "1215162010367852618";
        let _permission_id = "1215170943857885185";
        let app_id = "1020021532740116481";
        let cus_menu_id = "1192823941250891835";
        let sub_form_id = "1215162130668879876";

        // Step 1: 初始化流程
        info!("[API] Step 1: 初始化流程...");
        let body = json!({
            "cusMenuId": cus_menu_id,
            "urlPageTitle": "5a6e6ZmF5bel5pe2",
            "isCreate": 1,
            "workflowId": workflow_id,
            "id": workflow_id,
            "fieldAssignKeys": "",
            "jumplinkParamKey": "SubmitApplication",
        });

        let data = self.make_request("POST", "/api/workflow/core/flowPage/loadBaseParam", &body, eteamsid).await?;

        if data.get("status") == Some(&Value::Bool(false)) {
            let error_msg = data.get("msg").and_then(|v| v.as_str()).unwrap_or("loadBaseParam 失败");
            error!("[API] Step 1 失败: {}", error_msg);
            return Ok(SubmitResponse {
                success: false,
                message: error_msg.to_string(),
                request_name: None,
                result_message: None,
                record_id: None,
            });
        }

        let request_id = data["data"]["requestInfo"]["requestId"].as_str().unwrap_or("");
        let auth_str = data["data"]["commonParam"]["authStr"].as_str().unwrap_or("");
        let auth_sig = data["data"]["commonParam"]["authSignatureStr"].as_str().unwrap_or("");
        let page_key = format!("CreateDialog_{}_{}", workflow_id, chrono::Utc::now().timestamp_millis());
        info!("[API] Step 1 完成: requestId={}", request_id);

        // Step 2: 保存表单数据
        info!("[API] Step 2: 保存表单数据...");
        let mut data_details = vec![
            json!({"formField": {"id": "1217668209634082818"}, "content": work_date}),
            json!({"formField": {"id": "1215162130668879873"}, "dataOptions": [{"optionId": employee_id, "content": employee_name}]}),
            json!({"formField": {"id": "1215162130668879874"}, "content": work_date}),
            json!({"formField": {"id": "1216229352757690370"}, "dataOptions": [{"optionId": entries[0].project_id, "content": entries[0].project_name}]}),
        ];

        for (idx, entry) in entries.iter().enumerate() {
            let row_id = format!("{}", rand::random::<u64>() % 100_000_000_000_000_000 + 400_000_000_000_000_000);
            let time_slot_name = match entry.time_slot_id.as_str() {
                "0" => "10:30",
                "1" => "13:40",
                "2" => "15:40",
                "3" => "17:40",
                _ => "10:30",
            };

            data_details.push(json!({"dataIndex": idx + 1, "rowId": row_id, "subForm": {"id": sub_form_id}, "formField": {"id": "1215162323950796800"}, "dataOptions": [{"optionId": entry.time_slot_id, "content": time_slot_name}]}));
            data_details.push(json!({"dataIndex": idx + 1, "rowId": row_id, "subForm": {"id": sub_form_id}, "formField": {"id": "1215162323950796801"}, "dataOptions": [{"optionId": entry.project_id, "content": entry.project_name}]}));
            data_details.push(json!({"dataIndex": idx + 1, "rowId": row_id, "subForm": {"id": sub_form_id}, "formField": {"id": "1215162323950796802"}, "content": entry.hours}));
            data_details.push(json!({"dataIndex": idx + 1, "rowId": row_id, "subForm": {"id": sub_form_id}, "formField": {"id": "1227354155555807233"}, "content": work_date}));
            data_details.push(json!({"dataIndex": idx + 1, "rowId": row_id, "subForm": {"id": sub_form_id}, "formField": {"id": "1249973841324212226"}, "dataOptions": [{"optionId": entry.work_type_id, "content": entry.work_type_name}]}));
        }

        let body = json!({
            "module": "ebuildercard",
            "customParam": {
                "isCreate": true,
                "requestId": request_id,
                "isTest": false,
                "secretLevel": "0",
                "secretLevelValidity": "",
                "nodeId": node_id,
                "workflowId": workflow_id,
                "type": 2,
                "authSignatureStr": auth_sig,
                "authStr": auth_str,
                "identityId": employee_id,
                "identityType": 0,
                "monitorEditFormData": false,
            },
            "employeeId": employee_id,
            "appId": app_id,
            "formData": {
                "module": "ebuildercard",
                "presetDataId": request_id,
                "dataDetails": data_details,
                "client": "pc",
                "dataStatus": "submit",
                "form": {"id": form_id},
                "formLayout": {"id": form_layout_id},
            },
        });

        let _data = self.make_request("POST", &format!("/api/ebuilder/flow/form/core/saveFormData?formId={}", form_id), &body, eteamsid).await?;
        info!("[API] Step 2 完成: 表单数据已保存");

        // Step 3: 创建流程草稿
        info!("[API] Step 3: 创建流程草稿...");
        let body = json!({
            "cusMenuId": cus_menu_id,
            "urlPageTitle": "5a6e6ZmF5bel5pe2",
            "isCreate": true,
            "workflowId": workflow_id,
            "id": workflow_id,
            "fieldAssignKeys": "",
            "jumplinkParamKey": "SubmitApplication",
            "apiModule": "workflow",
            "identityId": employee_id,
            "identityType": 0,
            "requestId": request_id,
            "userCurrentNodeId": node_id,
            "isAgent": false,
            "beAgentId": "0",
            "fixedNodeId": node_id,
            "authStr": auth_str,
            "authSignatureStr": auth_sig,
            "isTest": false,
            "pageKey": page_key,
            "secLevel": "0",
            "secValidity": "",
            "needWfBack": true,
            "sameOperateId": null,
            "customBtnId": null,
            "confirmSecondSubmit": false,
            "selectFlowParam": null,
            "freeChoiceNodeIds": null,
            "dataId": request_id,
            "recordId": "",
            "voteSubmit": false,
            "unNeedVerifyPrivacyAgreement": 1,
            "remark": "",
            "attachments": [],
            "fileData": [],
            "locationData": {},
            "comment": {
                "relevanceList": [],
                "imAtContent": "",
                "originContent": "",
                "atLinkList": [],
                "content": "_weaverMte_",
            },
            "quoteData": [],
            "fullTextAnnotationData": [],
            "signatureData": {"signatureImgId": ""},
            "userSignatureData": {"signatureImgId": ""},
            "secondVerifySet": {
                "applyEnableViewAuth": false,
                "applyViewVerfiyType": 0,
                "applyEnableHandleAuth": false,
                "applyHandleVerifyType": 0,
                "applyVerifyOperation": "0,1,11,4,5",
                "applyHandleQysServer": "-1",
                "applyEnableDataProtect": false,
                "applyDataProtectMode": 0,
                "applyDataInteractMode": -1,
                "applyQysServer": "-1",
                "applyDataVerifyOperation": "0,1,11,4,5",
                "nodeEnableViewAuth": false,
                "nodeEnableDoubleAuth": false,
                "nodeAuthVerifier": "-1",
                "nodeEnableHandleAuth": false,
                "nodeEnableDataProtect": false,
                "nodeFieldDataProtect": [],
                "enableHandDoubleAuth": false,
                "handAuthVerifier": "-1",
                "nodeEnableSignatures": false,
            },
            "src": "save",
            "clientType": 0,
        });

        let data = self.make_request("POST", "/api/workflow/core/flow/create", &body, eteamsid).await?;

        if data.get("status") == Some(&Value::Bool(false)) {
            let error_msg = data.get("msg").and_then(|v| v.as_str()).unwrap_or("flow/create 失败");
            error!("[API] Step 3 失败: {}", error_msg);
            return Ok(SubmitResponse {
                success: false,
                message: error_msg.to_string(),
                request_name: None,
                result_message: None,
                record_id: None,
            });
        }

        let record_id = data.get("recordId").and_then(|v| v.as_str()).map(|s| s.to_string());
        info!("[API] Step 3 完成: recordId={:?}", record_id);

        // Step 4: 正式提交
        info!("[API] Step 4: 正式提交...");
        let body = json!({
            "operResultType": "SUCCESS_RELOAD_CURRENT_PAGE",
            "requestId": request_id,
            "userSpecifiedNodeId": 0,
            "needAutoOpen": 1,
            "isCreate": false,
            "apiModule": "workflow",
            "identityId": employee_id,
            "identityType": 0,
            "workflowId": workflow_id,
            "userCurrentNodeId": node_id,
            "isAgent": false,
            "beAgentId": "0",
            "fixedNodeId": node_id,
            "authStr": auth_str,
            "authSignatureStr": auth_sig,
            "isTest": false,
            "pageKey": page_key,
            "secLevel": "0",
            "secValidity": "",
            "needWfBack": true,
            "sameOperateId": null,
            "customBtnId": null,
            "confirmSecondSubmit": false,
            "selectFlowParam": null,
            "freeChoiceNodeIds": null,
            "testParam": {},
            "src": "submit",
            "voteSubmit": false,
            "seconedSubmit": false,
            "remark": "",
            "attachments": [],
            "fileData": [],
            "locationData": {},
            "comment": {
                "relevanceList": [],
                "imAtContent": "",
                "originContent": "",
                "atLinkList": [],
                "content": "_weaverMte_",
            },
            "quoteData": [],
            "fullTextAnnotationData": [],
            "signatureData": {"signatureImgId": ""},
            "userSignatureData": {"signatureImgId": ""},
            "unNeedVerifyPrivacyAgreement": 1,
            "clientType": 0,
        });

        let data = self.make_request("POST", &format!("/api/workflow/core/flow/submit?requestId={}", request_id), &body, eteamsid).await?;

        if data.get("status") == Some(&Value::Bool(false)) {
            let error_msg = data.get("msg").and_then(|v| v.as_str()).unwrap_or("flow/submit 失败");
            error!("[API] Step 4 失败: {}", error_msg);
            return Ok(SubmitResponse {
                success: false,
                message: error_msg.to_string(),
                request_name: None,
                result_message: None,
                record_id: None,
            });
        }

        info!("[API] 工时提交成功!");
        Ok(SubmitResponse {
            success: true,
            message: "工时填报成功".to_string(),
            request_name: data.get("requestName").and_then(|v| v.as_str()).map(|s| s.to_string()),
            result_message: data.get("resultMessage").and_then(|v| v.as_str()).map(|s| s.to_string()),
            record_id,
        })
    }
}

/// 判断方法是否为大写
fn method_upper(method: &str) -> bool {
    method.to_uppercase() == method
}
