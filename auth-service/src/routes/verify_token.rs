use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::domain::{AuthAPIError, BannedTokenStore, Email, UserStore};
use crate::model::verifytoken::VerifyTokenRequest;
use crate::utils::auth::validate_token;
use crate::AppState;

#[tracing::instrument(skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let t = match validate_token(&request.token).await {
        Ok(e) => e,
        Err(_e) => return Err(AuthAPIError::InvalidToken),
    };

    let email = Email::parse(t.sub).map_err(|_| AuthAPIError::MailformedToken)?;

    if state
        .banned_token_store
        .read()
        .await
        .is_banned(&request.token)
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?
    {
        return Err(AuthAPIError::InvalidToken);
    }

    if state.user_store.read().await.get_user(email).await.is_err() {
        return Err(AuthAPIError::InvalidToken);
    }

    Ok(StatusCode::OK.into_response())
}
