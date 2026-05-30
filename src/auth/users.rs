use axum::{extract::FromRequestParts, http::request::Parts};
use axum_extra::extract::CookieJar;
use jwt_simple::{
    algorithms::{HS256Key, MACLike},
    claims::Claims,
    reexports::coarsetime::Duration,
};
use password_auth::VerifyError;
use serde::{Deserialize, Serialize};

use crate::{app::AppState, errors::AppError, repository::Repository};

const SECRET_KEY: &[u8] = b"exorcise-secret-key";

fn auth_key() -> HS256Key {
    HS256Key::from_bytes(SECRET_KEY)
}

pub struct UnauthenticatedUser {
    username: String,
    password: String,
}

impl UnauthenticatedUser {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub async fn authenticate(&self, repository: &Repository) -> Result<User, AppError> {
        let user = match repository.get_user_by_username(&self.username).await? {
            Some(user) => user,
            None => return Err(AppError::InvalidCredentials),
        };

        match password_auth::verify_password(&self.password, &user.password_hash) {
            Ok(_) => Ok(User::new(user.id, user.username)),
            Err(VerifyError::PasswordInvalid) => Err(AppError::InvalidCredentials),
            Err(VerifyError::Parse(err)) => panic!("Hashing algorithm failed: {err}"),
        }
    }

    pub async fn register(&self, repository: &Repository) -> Result<User, AppError> {
        let password_hash = password_auth::generate_hash(&self.password);
        let user = match repository.create_user(&self.username, &password_hash).await {
            Ok(user) => user,
            Err(sqlx::Error::Database(db_error)) if db_error.is_unique_violation() => {
                return Err(AppError::UsernameTaken);
            }
            Err(err) => return Err(AppError::Database(err)),
        };

        Ok(User::new(user.id, user.username))
    }
}

pub struct User {
    id: i64,
    pub username: String,
}

#[derive(Serialize, Deserialize)]
struct UserClaims {
    id: i64,
    username: String,
}

impl From<User> for UserClaims {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
        }
    }
}

impl User {
    pub fn new(id: i64, username: String) -> Self {
        Self { id, username }
    }

    #[allow(dead_code)]
    pub const fn username(&self) -> &String {
        &self.username
    }

    pub const fn id(&self) -> i64 {
        self.id
    }

    pub fn auth_token(self) -> Result<String, AppError> {
        let claims = Claims::with_custom_claims(UserClaims::from(self), Duration::from_mins(10));
        let token = auth_key().authenticate(claims)?;

        Ok(token)
    }

    pub fn from_auth_token(token: &str) -> Result<Self, AppError> {
        let claims = auth_key().verify_token::<UserClaims>(token, None)?.custom;

        Ok(User::new(claims.id, claims.username))
    }
}

impl FromRequestParts<AppState> for User {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = match jar.get("token") {
            Some(token) => token.value(),
            None => return Err(AppError::MissingAuthorization),
        };

        User::from_auth_token(token)
    }
}
