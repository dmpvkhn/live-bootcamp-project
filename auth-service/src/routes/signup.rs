use crate::domain::AuthAPIError;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

use crate::{domain::User, model::signup::SignUPResponse, services::UserStoreError, AppState};

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let user = User {
        email: request.email,
        password: request.password,
        requires_2fa: request.requires_2fa,
    };

    if user.email.is_empty() || !user.email.contains("@") {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let mut user_store = state.user_store.write().await;
    match user_store.add_user(user) {
        Ok(_) => {}
        Err(e) => {
            match e {
                UserStoreError::UserAlreadyExists => return Err(AuthAPIError::UserAlreadyExists), //(StatusCode::CONFLICT, response)
                _ => return Err(AuthAPIError::UnexpectedError), //(StatusCode::INTERNAL_SERVER_ERROR, response),
            }
        }
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
