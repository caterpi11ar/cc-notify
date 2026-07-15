use serde_json::Value;

pub fn request_model(request: &Value) -> String {
    request
        .get("model")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

pub fn request_stream(request: &Value) -> bool {
    request
        .get("stream")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub fn estimate_tokens(request: &Value) -> i64 {
    request.to_string().len() as i64 / 4
}
