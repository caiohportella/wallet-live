use axum::{
    Json,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Missing Authorization Headers")]
    MissingAuthorization,

    #[error("Invalid Credentials")]
    InvalidCredentials,

    #[error("Asset Does Not Exist")]
    AssetDoesNotExist,

    #[error("Username Taken")]
    UsernameTaken,

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Template(#[from] askama::Error),

    #[error(transparent)]
    Jwt(#[from] jwt_simple::Error),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    code: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::MissingAuthorization => axum::http::StatusCode::BAD_REQUEST,
            Self::InvalidCredentials => axum::http::StatusCode::UNAUTHORIZED,
            Self::AssetDoesNotExist => axum::http::StatusCode::NOT_FOUND,
            Self::UsernameTaken => axum::http::StatusCode::CONFLICT,
            Self::Database(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::Template(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::Jwt(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error_response = ErrorResponse {
            code: status.as_u16(),
            message: self.to_string(),
        };

        (status, Json(error_response)).into_response()
    }
}
