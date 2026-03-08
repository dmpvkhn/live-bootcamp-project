use crate::domain::TwoFACodeStore;
use axum::response::IntoResponse;
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::CookieJar;

use crate::domain::{AuthAPIError, Email, HashedPassword, LoginAttemptId, TwoFACode, UserStore};
use crate::model::login::{LoginRequest, LoginResponse, TwoFactorAuthResponse};
use crate::utils::auth::generate_auth_cookie;
use crate::AppState;

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    // let result = async {
    let email = match Email::parse(request.email) {
        Ok(e) => e,
        Err(_e) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    let user_store = &state.user_store.read().await;

    match user_store
        .validate_user(email.clone(), &request.password)
        .await
    {
        Ok(_) => {}
        Err(_e) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    }

    let user = match user_store.get_user(email.clone()).await {
        Ok(u) => u,
        Err(_e) => return (jar, Err(AuthAPIError::IncorrectCredentials)),
    };

    match user.requires_2fa {
        true => handle_2fa(&user.email, &state, jar).await,
        false => handle_no_2fa(&user.email, jar).await,
    }
}

async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();

    if state
        .two_fa_code_store
        .write()
        .await
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::UnexpectedError));
    };

    if state
        .email_client
        .send_email(email, "2FA Code", two_fa_code.as_ref())
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }

    let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
        message: "2FA required".to_owned(),
        login_attempt_id: login_attempt_id.as_ref().to_owned(),
    }));
    (jar, Ok((StatusCode::PARTIAL_CONTENT, response)))
}

// New!
async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let result = async {
        let auth_cookie =
            generate_auth_cookie(email).map_err(|_e| AuthAPIError::UnexpectedError)?;
        Ok(auth_cookie)
    }
    .await;

    match result {
        Ok(auth_cookie) => {
            let updated_jar = jar.add(auth_cookie);

            (
                updated_jar,
                Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))),
            )
        }
        Err(e) => (jar, (Err(e))),
    }
}
