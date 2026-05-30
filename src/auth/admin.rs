use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use crate::app::AppState;
use crate::errors::AppError;

const ADMIN_SECRET_KEY: &str = "admin";

pub struct Admin;

fn is_valid_admin_header(value: &str) -> bool {
    let normalized = value.trim().trim_matches('"');
    let mut parts = normalized.split_whitespace();

    match (parts.next(), parts.next(), parts.next()) {
        (Some(secret), None, None) => secret == ADMIN_SECRET_KEY,
        (Some(scheme), Some(secret), None) => {
            scheme.eq_ignore_ascii_case("bearer") && secret == ADMIN_SECRET_KEY
        }
        _ => false,
    }
}

impl FromRequestParts<AppState> for Admin {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _: &AppState) -> Result<Self, Self::Rejection> {
        let Some(auth_header) = parts.headers.get(AUTHORIZATION) else {
            return Err(AppError::MissingAuthorization);
        };

        let Some(auth) = auth_header.to_str().ok() else {
            return Err(AppError::InvalidCredentials);
        };

        if !is_valid_admin_header(auth) {
            return Err(AppError::InvalidCredentials);
        }

        Ok(Admin)
    }
}
