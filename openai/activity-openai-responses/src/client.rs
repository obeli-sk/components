use crate::exports::obelisk_components::openai_responses::api::{
    CreateResponseRequest, ListInputItemsResult, ListResponsesResult, Response, SimpleRequest,
};
use crate::obelisk_components::openai_responses::types::{
    Annotation, ApiError, ApiErrorDetails, FileCitation, FilePath, FunctionCallOutput, ImageDetail,
    IncompleteDetails, InputContent, InputItem, InputMessage, InputTokensDetails, ItemStatus,
    OutputContent, OutputItem, OutputMessage, OutputText, OutputTokensDetails, ReasoningOutput,
    ReasoningSummaryItem, ResponseError, ResponseStatus, Role, SearchContextSize, Tool, Truncation,
    UrlCitation, Usage, WebSearchOutput,
};
use serde::{Deserialize, Serialize};
use wstd::http::{Body, Client, HeaderValue, Request};

const API_BASE: &str = "https://api.openai.com/v1";

fn get_api_key() -> Result<String, ApiError> {
    std::env::var("OPENAI_API_KEY").map_err(|_| {
        ApiError::ConfigurationError("OPENAI_API_KEY environment variable not set".into())
    })
}

fn request_error(msg: &str) -> ApiError {
    ApiError::RequestFailed(msg.into())
}

fn parse_error(msg: &str) -> ApiError {
    ApiError::ParseError(msg.into())
}

#[derive(Serialize)]
struct JsonRequest {
    model: String,
    input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Deserialize)]
struct JsonResponse {
    id: String,
    object: String,
    created_at: u64,
    model: String,
    output: Vec<serde_json::Value>,
    status: String,
    #[serde(default)]
    usage: Option<JsonUsage>,
    #[serde(default)]
    error: Option<JsonError>,
    #[serde(default)]
    incomplete_details: Option<JsonIncompleteDetails>,
    #[serde(default)]
    instructions: Option<String>,
    #[serde(default)]
    metadata: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    parallel_tool_calls: bool,
    #[serde(default)]
    previous_response_id: Option<String>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    top_p: Option<f32>,
    #[serde(default)]
    truncation: Option<String>,
    #[serde(default)]
    user: Option<String>,
}

#[derive(Deserialize)]
struct JsonUsage {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
    #[serde(default)]
    input_tokens_details: Option<JsonInputTokensDetails>,
    #[serde(default)]
    output_tokens_details: Option<JsonOutputTokensDetails>,
}

#[derive(Deserialize)]
struct JsonInputTokensDetails {
    cached_tokens: u32,
}

#[derive(Deserialize)]
struct JsonOutputTokensDetails {
    reasoning_tokens: u32,
}

#[derive(Deserialize)]
struct JsonError {
    code: String,
    message: String,
}

#[derive(Deserialize)]
struct JsonIncompleteDetails {
    reason: String,
}

#[derive(Deserialize)]
struct JsonApiError {
    error: JsonApiErrorInner,
}

#[derive(Deserialize)]
struct JsonApiErrorInner {
    code: Option<String>,
    message: String,
    param: Option<String>,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Deserialize)]
struct JsonListResponse {
    object: String,
    data: Vec<JsonResponse>,
    has_more: bool,
    first_id: Option<String>,
    last_id: Option<String>,
}

#[derive(Deserialize)]
struct JsonListInputItems {
    object: String,
    data: Vec<JsonInputItem>,
    has_more: bool,
    first_id: Option<String>,
    last_id: Option<String>,
}

#[derive(Deserialize)]
struct JsonInputItem {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    #[serde(default)]
    content: Option<serde_json::Value>,
}

fn convert_response(jr: JsonResponse) -> Response {
    Response {
        id: jr.id,
        object: jr.object,
        created_at: jr.created_at,
        model: jr.model,
        output: jr.output.into_iter().map(convert_output_item).collect(),
        status: match jr.status.as_str() {
            "in_progress" => ResponseStatus::InProgress,
            "completed" => ResponseStatus::Completed,
            "incomplete" => ResponseStatus::Incomplete,
            "failed" => ResponseStatus::Failed,
            "cancelled" => ResponseStatus::Cancelled,
            _ => ResponseStatus::Failed,
        },
        usage: jr.usage.map(|u| Usage {
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
            total_tokens: u.total_tokens,
            input_tokens_details: u.input_tokens_details.map(|d| InputTokensDetails {
                cached_tokens: d.cached_tokens,
            }),
            output_tokens_details: u.output_tokens_details.map(|d| OutputTokensDetails {
                reasoning_tokens: d.reasoning_tokens,
            }),
        }),
        error: jr.error.map(|e| ResponseError {
            code: e.code,
            message: e.message,
        }),
        incomplete_details: jr
            .incomplete_details
            .map(|d| IncompleteDetails { reason: d.reason }),
        instructions: jr.instructions,
        metadata: jr.metadata.map(|m| m.into_iter().collect()),
        parallel_tool_calls: jr.parallel_tool_calls,
        previous_response_id: jr.previous_response_id,
        temperature: jr.temperature,
        top_p: jr.top_p,
        truncation: jr.truncation.map(|t| match t.as_str() {
            "auto" => Truncation::Auto,
            _ => Truncation::Disabled,
        }),
        user: jr.user,
    }
}

fn convert_output_item(v: serde_json::Value) -> OutputItem {
    let type_ = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match type_ {
        "message" => {
            let content = v
                .get("content")
                .and_then(|c| c.as_array())
                .map(|arr| arr.iter().map(convert_output_content).collect())
                .unwrap_or_default();
            OutputItem::Message(OutputMessage {
                id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").into(),
                role: match v.get("role").and_then(|r| r.as_str()).unwrap_or("") {
                    "user" => Role::User,
                    "system" => Role::System,
                    "developer" => Role::Developer,
                    _ => Role::Assistant,
                },
                content,
                status: convert_item_status(v.get("status").and_then(|s| s.as_str())),
            })
        }
        "function_call" => OutputItem::FunctionCall(FunctionCallOutput {
            id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").into(),
            call_id: v
                .get("call_id")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .into(),
            name: v.get("name").and_then(|x| x.as_str()).unwrap_or("").into(),
            arguments: v
                .get("arguments")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .into(),
            status: convert_item_status(v.get("status").and_then(|s| s.as_str())),
        }),
        "web_search_call" => OutputItem::WebSearchCall(WebSearchOutput {
            id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").into(),
            status: convert_item_status(v.get("status").and_then(|s| s.as_str())),
        }),
        "reasoning" => OutputItem::Reasoning(ReasoningOutput {
            id: v.get("id").and_then(|x| x.as_str()).unwrap_or("").into(),
            summary: v
                .get("summary")
                .and_then(|s| s.as_array())
                .map(|arr| {
                    arr.iter()
                        .map(|item| ReasoningSummaryItem {
                            type_: item
                                .get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("")
                                .into(),
                            text: item
                                .get("text")
                                .and_then(|t| t.as_str())
                                .unwrap_or("")
                                .into(),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            status: convert_item_status(v.get("status").and_then(|s| s.as_str())),
        }),
        _ => OutputItem::Message(OutputMessage {
            id: String::new(),
            role: Role::Assistant,
            content: vec![],
            status: ItemStatus::Completed,
        }),
    }
}

fn convert_output_content(v: &serde_json::Value) -> OutputContent {
    let type_ = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    match type_ {
        "refusal" => OutputContent::Refusal(
            v.get("refusal")
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .into(),
        ),
        _ => OutputContent::Text(OutputText {
            text: v.get("text").and_then(|t| t.as_str()).unwrap_or("").into(),
            annotations: v
                .get("annotations")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().filter_map(convert_annotation).collect())
                .unwrap_or_default(),
        }),
    }
}

fn convert_annotation(v: &serde_json::Value) -> Option<Annotation> {
    let type_ = v.get("type").and_then(|t| t.as_str())?;
    match type_ {
        "url_citation" => Some(Annotation::UrlCitation(UrlCitation {
            url: v.get("url").and_then(|u| u.as_str()).unwrap_or("").into(),
            title: v.get("title").and_then(|t| t.as_str()).unwrap_or("").into(),
            start_index: v.get("start_index").and_then(|i| i.as_u64()).unwrap_or(0) as u32,
            end_index: v.get("end_index").and_then(|i| i.as_u64()).unwrap_or(0) as u32,
        })),
        "file_citation" => Some(Annotation::FileCitation(FileCitation {
            file_id: v
                .get("file_id")
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .into(),
            index: v.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as u32,
        })),
        "file_path" => Some(Annotation::FilePath(FilePath {
            file_id: v
                .get("file_id")
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .into(),
            index: v.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as u32,
        })),
        _ => None,
    }
}

fn convert_item_status(s: Option<&str>) -> ItemStatus {
    match s {
        Some("in_progress") => ItemStatus::InProgress,
        Some("incomplete") => ItemStatus::Incomplete,
        _ => ItemStatus::Completed,
    }
}

async fn do_request(method: &str, path: &str, body: Option<&str>) -> Result<Vec<u8>, ApiError> {
    let api_key = get_api_key()?;
    let url = format!("{}{}", API_BASE, path);

    let req = match method {
        "GET" => Request::get(&url),
        "DELETE" => Request::delete(&url),
        _ => Request::post(&url),
    };

    let req = req
        .header(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| request_error(&e.to_string()))?,
        )
        .header("Content-Type", HeaderValue::from_static("application/json"));

    let req = if let Some(b) = body {
        req.body(Body::from(b.to_string()))
            .map_err(|e| request_error(&e.to_string()))?
    } else {
        req.body(Body::empty())
            .map_err(|e| request_error(&e.to_string()))?
    };

    let resp = Client::new()
        .send(req)
        .await
        .map_err(|e| request_error(&e.to_string()))?;
    let status = resp.status();
    let mut body = resp.into_body();
    let bytes = body
        .contents()
        .await
        .map_err(|e| request_error(&e.to_string()))?;

    if !status.is_success() {
        if let Ok(err) = serde_json::from_slice::<JsonApiError>(bytes) {
            return Err(ApiError::ApiError(ApiErrorDetails {
                code: err.error.code.unwrap_or_default(),
                message: err.error.message,
                param: err.error.param,
                type_: err.error.type_,
            }));
        }
        return Err(request_error(&format!(
            "HTTP {}: {}",
            status.as_u16(),
            String::from_utf8_lossy(bytes)
        )));
    }

    Ok(bytes.to_vec())
}

fn convert_input_message(msg: &InputMessage) -> serde_json::Value {
    let role = match msg.role {
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::System => "system",
        Role::Developer => "developer",
    };
    let content: Vec<serde_json::Value> = msg
        .content
        .iter()
        .map(|c| match c {
            InputContent::Text(t) => serde_json::json!({ "type": "input_text", "text": t }),
            InputContent::ImageUrl(img) => serde_json::json!({
                "type": "input_image",
                "image_url": img.url,
                "detail": img.detail.as_ref().map(|d| match d {
                    ImageDetail::Low => "low",
                    ImageDetail::High => "high",
                    ImageDetail::Auto => "auto",
                })
            }),
            InputContent::ImageFile(img) => serde_json::json!({
                "type": "input_image",
                "file_id": img.file_id,
                "detail": img.detail.as_ref().map(|d| match d {
                    ImageDetail::Low => "low",
                    ImageDetail::High => "high",
                    ImageDetail::Auto => "auto",
                })
            }),
        })
        .collect();
    serde_json::json!({ "role": role, "content": content })
}

fn convert_tool(tool: &Tool) -> serde_json::Value {
    match tool {
        Tool::Function(f) => serde_json::json!({
            "type": "function",
            "name": f.name,
            "description": f.description,
            "parameters": f.parameters.as_ref().and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok()),
            "strict": f.strict,
        }),
        Tool::WebSearch(w) => serde_json::json!({
            "type": "web_search_preview",
            "search_context_size": w.search_context_size.as_ref().map(|s| match s {
                SearchContextSize::Low => "low",
                SearchContextSize::Medium => "medium",
                SearchContextSize::High => "high",
            }),
        }),
        Tool::FileSearch(f) => serde_json::json!({
            "type": "file_search",
            "vector_store_ids": f.vector_store_ids,
            "max_num_results": f.max_num_results,
        }),
        Tool::CodeInterpreter(c) => serde_json::json!({
            "type": "code_interpreter",
            "container": c.container,
        }),
        Tool::ComputerUse(c) => serde_json::json!({
            "type": "computer_use_preview",
            "environment": c.environment,
            "display_width": c.display_width,
            "display_height": c.display_height,
        }),
    }
}

pub async fn create_response(request: CreateResponseRequest) -> Result<Response, ApiError> {
    let input: Vec<serde_json::Value> = request.input.iter().map(convert_input_message).collect();
    let tools: Option<Vec<serde_json::Value>> = request
        .tools
        .as_ref()
        .map(|t| t.iter().map(convert_tool).collect());

    let req = JsonRequest {
        model: request.model,
        input: serde_json::Value::Array(input),
        instructions: request.instructions,
        max_output_tokens: request.max_output_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        store: request.store,
        metadata: request.metadata.map(|m| m.into_iter().collect()),
        tools,
    };

    let body = serde_json::to_string(&req).map_err(|e| parse_error(&e.to_string()))?;
    let bytes = do_request("POST", "/responses", Some(&body)).await?;
    let jr: JsonResponse =
        serde_json::from_slice(&bytes).map_err(|e| parse_error(&e.to_string()))?;
    Ok(convert_response(jr))
}

pub async fn create_simple_response(request: SimpleRequest) -> Result<Response, ApiError> {
    let tools: Option<Vec<serde_json::Value>> = request
        .tools
        .as_ref()
        .map(|t| t.iter().map(convert_tool).collect());

    let req = JsonRequest {
        model: request.model,
        input: serde_json::Value::String(request.input),
        instructions: request.instructions,
        max_output_tokens: request.max_output_tokens,
        temperature: request.temperature,
        top_p: None,
        store: None,
        metadata: None,
        tools,
    };

    let body = serde_json::to_string(&req).map_err(|e| parse_error(&e.to_string()))?;
    let bytes = do_request("POST", "/responses", Some(&body)).await?;
    let jr: JsonResponse =
        serde_json::from_slice(&bytes).map_err(|e| parse_error(&e.to_string()))?;
    Ok(convert_response(jr))
}

pub async fn get_response(response_id: &str) -> Result<Response, ApiError> {
    let bytes = do_request("GET", &format!("/responses/{}", response_id), None).await?;
    let jr: JsonResponse =
        serde_json::from_slice(&bytes).map_err(|e| parse_error(&e.to_string()))?;
    Ok(convert_response(jr))
}

pub async fn delete_response(response_id: &str) -> Result<bool, ApiError> {
    do_request("DELETE", &format!("/responses/{}", response_id), None).await?;
    Ok(true)
}

pub async fn list_responses(
    limit: Option<u32>,
    order: Option<String>,
    after: Option<String>,
    before: Option<String>,
) -> Result<ListResponsesResult, ApiError> {
    let mut params = vec![];
    if let Some(l) = limit {
        params.push(format!("limit={}", l));
    }
    if let Some(o) = order {
        params.push(format!("order={}", o));
    }
    if let Some(a) = after {
        params.push(format!("after={}", a));
    }
    if let Some(b) = before {
        params.push(format!("before={}", b));
    }
    let qs = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };

    let bytes = do_request("GET", &format!("/responses{}", qs), None).await?;
    let jr: JsonListResponse =
        serde_json::from_slice(&bytes).map_err(|e| parse_error(&e.to_string()))?;
    Ok(ListResponsesResult {
        object: jr.object,
        data: jr.data.into_iter().map(convert_response).collect(),
        has_more: jr.has_more,
        first_id: jr.first_id,
        last_id: jr.last_id,
    })
}

pub async fn list_input_items(
    response_id: &str,
    limit: Option<u32>,
    order: Option<String>,
    after: Option<String>,
    before: Option<String>,
) -> Result<ListInputItemsResult, ApiError> {
    let mut params = vec![];
    if let Some(l) = limit {
        params.push(format!("limit={}", l));
    }
    if let Some(o) = order {
        params.push(format!("order={}", o));
    }
    if let Some(a) = after {
        params.push(format!("after={}", a));
    }
    if let Some(b) = before {
        params.push(format!("before={}", b));
    }
    let qs = if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    };

    let bytes = do_request(
        "GET",
        &format!("/responses/{}/input_items{}", response_id, qs),
        None,
    )
    .await?;
    let jr: JsonListInputItems =
        serde_json::from_slice(&bytes).map_err(|e| parse_error(&e.to_string()))?;
    Ok(ListInputItemsResult {
        object: jr.object,
        data: jr
            .data
            .into_iter()
            .map(|i| InputItem {
                id: i.id,
                type_: i.type_,
                content: i.content.map(|c| c.to_string()),
            })
            .collect(),
        has_more: jr.has_more,
        first_id: jr.first_id,
        last_id: jr.last_id,
    })
}

pub async fn cancel_response(response_id: &str) -> Result<Response, ApiError> {
    let bytes = do_request("POST", &format!("/responses/{}/cancel", response_id), None).await?;
    let jr: JsonResponse =
        serde_json::from_slice(&bytes).map_err(|e| parse_error(&e.to_string()))?;
    Ok(convert_response(jr))
}
