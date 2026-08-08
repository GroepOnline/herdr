use crate::api::schema::{ErrorBody, ErrorResponse, ResponseResult, SuccessResponse};

pub(super) fn encode_success(id: String, result: ResponseResult) -> String {
    serde_json::to_string(&SuccessResponse { id, result })
        .unwrap_or_else(|error| format!(r#"{{"error":"response serialization failed: {error}"}}"#))
}

pub(super) fn encode_error(id: String, code: &str, message: impl Into<String>) -> String {
    encode_error_body(
        id,
        ErrorBody {
            code: code.into(),
            message: message.into(),
        },
    )
}

pub(super) fn encode_error_body(id: String, error: ErrorBody) -> String {
    serde_json::to_string(&ErrorResponse { id, error })
        .unwrap_or_else(|error| format!(r#"{{"error":"response serialization failed: {error}"}}"#))
}
