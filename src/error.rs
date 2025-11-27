use axum::{http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn error_response(
    status: axum::http::StatusCode,
    error_type: ErrorTypes,
    error_msg: &str,
) -> axum::response::Response {
    (
        status,
        axum::Json(ErrorResponse::new(error_type, error_msg)),
    )
        .into_response()
}
#[macro_export]
macro_rules! error_response {
    ($status:expr, $error_type:expr, $($arg:tt)*) => {
        crate::error::error_response($status, $error_type, &format!($($arg)*))
    };
}

pub struct AppError(pub anyhow::Error);
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        error_response!(
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorTypes::InternalError,
            "Something went wrong: {}",
            self.0
        )
    }
}
impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

// Errors stuff
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error_type: String,
    pub error_msg: String,
}
impl ErrorResponse {
    pub fn new(error_type: ErrorTypes, error_msg: &str) -> Self {
        Self {
            error_type: error_type.as_str().to_string(),
            error_msg: error_msg.to_owned(),
        }
    }
}

#[derive(Debug)]
pub enum ErrorTypes {
    InternalError,
    BadData,
}

impl ErrorTypes {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorTypes::InternalError => "internal_error",
            ErrorTypes::BadData => "bad_data",
        }
    }
}
