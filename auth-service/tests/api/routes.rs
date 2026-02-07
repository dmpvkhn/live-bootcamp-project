use auth_service::model::{
    login::LoginRequest, verify2fa::VerifyRequest, verifytoken::VerifyTokenRequest,
};

use crate::helpers::TestApp;

#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;

    let response = app.get_root().await;

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.headers().get("content-type").unwrap(), "text/html");
}

#[tokio::test]
async fn login_simple_request() {
    let app = TestApp::new().await;

    let request = LoginRequest {
        email: String::from("admin@example.com"),
        password: String::from("123456"),
    };

    let response = app.post_login(request).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_two_fa_simple_request() {
    let app = TestApp::new().await;

    let request = VerifyRequest {
        email: String::from("admin@example.com"),
        login_attempt_id: String::from("id1234"),
        two_fa_code: String::from("123321"),
    };

    let response = app.post_verify_2fa(request).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn logout_request() {
    let app = TestApp::new().await;

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn verify_token_request() {
    let app = TestApp::new().await;

    let token_request = VerifyTokenRequest {
        token: String::from("token-123"),
    };

    let response = app.post_verify_token(token_request).await;

    assert_eq!(response.status().as_u16(), 200);
}
