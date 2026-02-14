use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie::Cookie, CookieJar};

use crate::{
    domain::{AuthAPIError, BannedTokenStore},
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
    AppState,
};

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let cookie = match jar.get(JWT_COOKIE_NAME) {
        Some(c) => c,
        None => return (jar, Err(AuthAPIError::InvalidCredentials)),
    };

    let token = cookie.value().to_owned();

    match validate_token(&token).await {
        Ok(_) => {}
        Err(_e) => return (jar, Err(AuthAPIError::InvalidToken)),
    }

    {
        let mut writer = state.banned_token_store.write().await;
        let _ = writer.store_token(token).await;
    }
    let jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));

    (jar, Ok(StatusCode::OK))
}
