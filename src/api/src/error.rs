use axum::{http::StatusCode, response::IntoResponse};

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("internal server error: {0}")]
    Unknown(#[from] anyhow::Error)
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Unknown(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                err.to_string()
            ).into_response()
        }
    }
}

pub type Result<T> = anyhow::Result<T, ApiError>;