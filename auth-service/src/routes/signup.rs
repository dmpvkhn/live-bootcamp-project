use crate::domain::AuthAPIError;
use crate::domain::Email;
use crate::domain::Password;
use crate::domain::UserStore;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{domain::User, domain::UserStoreError, model::signup::SignUPResponse, AppState};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let user = User {
        email: Email::parse(request.email).map_err(|_e| AuthAPIError::InvalidCredentials)?,
        password: Password::parse(request.password)
            .map_err(|_e| AuthAPIError::InvalidCredentials)?,
        requires_2fa: request.requires_2fa,
    };

    let user_store = state.user_store.clone();
    match user_store.write().await.add_user(user).await {
        Ok(_) => {}
        Err(e) => match e {
            UserStoreError::UserAlreadyExists => return Err(AuthAPIError::UserAlreadyExists),
            _ => return Err(AuthAPIError::UnexpectedError),
        },
    };

    // TODO: Add `user` to the `user_store`. Simply unwrap the returned `Result` enum type for now.

    let response = Json(SignUPResponse {
        message: "User created successfully!".to_string(),
    });

    Ok((StatusCode::CREATED, response))
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}
