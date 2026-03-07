use crate::domain::TwoFACodeStore;
use axum::response::IntoResponse;
use axum::Json;
use axum::{extract::State, http::StatusCode};
use axum_extra::extract::CookieJar;
use serde;
use serde::Deserialize;

use crate::domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode};
use crate::utils::auth::generate_auth_cookie;
use crate::AppState;

pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = match Email::parse(request.email) {
        Ok(e) => e,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let login_attempt_id = match LoginAttemptId::parse(request.login_attempt_id) {
        Ok(id) => id,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let two_fa_code = match TwoFACode::parse(request.two_fa_code) {
        Ok(code) => code,
        Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let mut store = state.two_fa_code_store.write().await;

    let (stored_id, stored_code) = match store.get_code(&email).await {
        Ok(pair) => pair,
        Err(_) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    if stored_id.as_ref() != login_attempt_id.as_ref()
        || stored_code.as_ref() != two_fa_code.as_ref()
    {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    if store.remove_code(&email).await.is_err() {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }

    let cookie = match generate_auth_cookie(&email) {
        Ok(c) => c,
        Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
    };
    (jar.add(cookie), Ok(StatusCode::OK.into_response()))
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    pub email: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
    #[serde(rename = "2FACode")]
    pub two_fa_code: String,
}
