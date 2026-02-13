use axum::response::IntoResponse;
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;

use crate::domain::{AuthAPIError, Email, Password, UserStore};
use crate::model::login::LoginRequest;
use crate::utils::auth::generate_auth_cookie;
use crate::AppState;

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let result = async {
        let email = Email::parse(request.email).map_err(|_e| AuthAPIError::IncorrectCredentials)?;
        let password =
            Password::parse(request.password).map_err(|_e| AuthAPIError::InvalidCredentials)?;

        let user_store = &state.user_store.read().await;

        user_store
            .validate_user(email.clone(), password)
            .await
            .map_err(|_e| AuthAPIError::IncorrectCredentials)?;

        let _user = user_store
            .get_user(email.clone())
            .await
            .map_err(|_e| AuthAPIError::IncorrectCredentials)?;

        let auth_cookie =
            generate_auth_cookie(&email).map_err(|_e| AuthAPIError::UnexpectedError)?;
        Ok(auth_cookie)
    }
    .await;

    match result {
        Ok(auth_cookie) => {
            let updated_jar = jar.add(auth_cookie);

            (updated_jar, Ok(StatusCode::OK.into_response()))
        }
        Err(e) => (jar, (Err(e))),
    }
}
