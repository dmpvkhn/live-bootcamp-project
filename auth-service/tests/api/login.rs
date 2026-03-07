use crate::helpers::{get_random_email, TestApp};
use auth_service::domain::TwoFACodeStore;
use auth_service::{
    model::login::TwoFactorAuthResponse, utils::constants::JWT_COOKIE_NAME, ErrorResponse,
};

#[tokio::test]
async fn should_return_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    let test_case = serde_json::json!({
        "email": "adminexample.com",
        "password": "password123",
    });

    let response = app.post_login(&test_case).await;

    assert_eq!(
        response.status().as_u16(),
        422,
        "Failed for input: {:?}",
        test_case
    );
}
#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app = TestApp::new().await;

    let test_case = serde_json::json!({
        // "email": "adminexample.com",
        "password": "password123",
    });

    let response = app.post_login(&test_case).await;

    assert_eq!(
        response.status().as_u16(),
        422,
        "Failed for input: {:?}",
        test_case
    );
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let app = TestApp::new().await;

    let test_case = serde_json::json!({
        "email": "admin@example.com",
        "password": "password123",
    });

    let response = app.post_login(&test_case).await;

    assert_eq!(
        response.status().as_u16(),
        422,
        "Failed for input: {:?}",
        test_case
    );
}

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 206);

    let json_body = response
        .json::<TwoFactorAuthResponse>()
        .await
        .expect("Could not deserialize response body to TwoFactorAuthResponse");

    assert_eq!(json_body.message, "2FA required".to_owned());

    let email = auth_service::domain::Email::parse(random_email).unwrap();
    let store = app.two_fa_code_store.read().await;
    let (stored_id, _) = store
        .get_code(&email)
        .await
        .expect("2FA code not found in store");
    assert_eq!(stored_id.as_ref(), json_body.login_attempt_id);
}
