use crate::helpers::{get_random_email, TestApp};
use auth_service::{
    domain::{Email, TwoFACodeStore},
    model::login::TwoFactorAuthResponse,
    utils::constants::JWT_COOKIE_NAME,
};
use secrecy::SecretString;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    app.post_signup(&serde_json::json!({
        "email": random_email, "password": "password123", "requires2FA": true
    }))
    .await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_body = app
        .post_login(&serde_json::json!({
            "email": random_email, "password": "password123",
        }))
        .await
        .json::<TwoFactorAuthResponse>()
        .await
        .unwrap();

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

    let body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_body.login_attempt_id,
        "2FACode": "000000"  // wrong code
    });
    assert_eq!(app.post_verify_2fa(&body).await.status().as_u16(), 401);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_old_code() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    app.post_signup(&serde_json::json!({
        "email": random_email, "password": "password123", "requires2FA": true
    }))
    .await;

    let login_body1 = app
        .post_login(&serde_json::json!({
            "email": random_email, "password": "password123",
        }))
        .await
        .json::<TwoFactorAuthResponse>()
        .await
        .unwrap();

    // Second login overwrites the store
    app.post_login(&serde_json::json!({
        "email": random_email, "password": "password123",
    }))
    .await;

    let email = Email::parse(SecretString::new(random_email.clone().into_boxed_str())).unwrap();
    let store = app.two_fa_code_store.read().await;
    let (_, current_code) = store.get_code(&email).await.unwrap();
    drop(store);

    // First login's ID + current code → mismatch
    let body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_body1.login_attempt_id,
        "2FACode": current_code.as_ref()
    });
    assert_eq!(app.post_verify_2fa(&body).await.status().as_u16(), 401);

    app.clean_up().await;
}
#[tokio::test]
async fn should_return_200_if_correct_code() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    app.post_signup(&serde_json::json!({
        "email": random_email, "password": "password123", "requires2FA": true
    }))
    .await;

    let login_body = app
        .post_login(&serde_json::json!({
            "email": random_email, "password": "password123",
        }))
        .await
        .json::<TwoFactorAuthResponse>()
        .await
        .unwrap();

    let email = Email::parse(SecretString::new(random_email.clone().into_boxed_str())).unwrap();
    let store = app.two_fa_code_store.read().await;
    let (_, code) = store.get_code(&email).await.unwrap();
    drop(store);

    let body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_body.login_attempt_id,
        "2FACode": code.as_ref()
    });
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|c| c.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let mut app = TestApp::new().await;
    let random_email = get_random_email();

    app.post_signup(&serde_json::json!({
        "email": random_email, "password": "password123", "requires2FA": true
    }))
    .await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let login_body = app
        .post_login(&serde_json::json!({
            "email": random_email, "password": "password123",
        }))
        .await
        .json::<TwoFactorAuthResponse>()
        .await
        .unwrap();

    let email = Email::parse(SecretString::new(random_email.clone().into_boxed_str())).unwrap();
    let store = app.two_fa_code_store.read().await;
    let (_, code) = store.get_code(&email).await.unwrap();
    drop(store);

    let body = serde_json::json!({
        "email": random_email,
        "loginAttemptId": login_body.login_attempt_id,
        "2FACode": code.as_ref()
    });

    assert_eq!(app.post_verify_2fa(&body).await.status().as_u16(), 200);
    assert_eq!(app.post_verify_2fa(&body).await.status().as_u16(), 401);

    app.clean_up().await;
}

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let mut app = TestApp::new().await;

    let body = serde_json::json!({ "invalid": "data" });
    let response = app.post_verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 422);

    app.clean_up().await;
}
#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let mut app = TestApp::new().await;

    let body = serde_json::json!({
        "email": "not-an-email",
        "loginAttemptId": "not-a-uuid",
        "2FACode": "abc"
    });
    let response = app.post_verify_2fa(&body).await;

    assert_eq!(response.status().as_u16(), 400);

    app.clean_up().await;
}
