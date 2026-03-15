use auth_service::domain::TwoFACodeStore;
use auth_service::get_redis_client;
use auth_service::model::*;
use auth_service::services::mock_email_client::MockEmailClient;
use auth_service::services::RedisBannedTokenStore;
use auth_service::utils::constants::test;
use auth_service::utils::constants::DATABASE_URL;
use auth_service::utils::constants::DEFAULT_REDIS_HOSTNAME;
use auth_service::AppState;
use auth_service::Application;
use auth_service::BannedStoreType;
use auth_service::TwoFACodeStoreType;
use auth_service::UserStoreType;
use auth_service::{get_postgres_pool, services::PostgresUserStore};
use redis::Connection as RedisConnection;
use reqwest::cookie::Jar;
use reqwest::Client;
use sqlx::postgres::PgConnectOptions;
use sqlx::Connection;
use sqlx::PgConnection;
use sqlx::{postgres::PgPoolOptions, Executor, PgPool};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub db_name: String,
    pub clean_up_called: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        let db_name = Uuid::new_v4().to_string();
        let pg_pool = configure_postgresql(&db_name).await;
        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let redis_conn = Arc::new(RwLock::new(configure_redis()));
        let banned_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_conn)));
        let two_fa_code_store = TwoFACodeStoreType::default();
        let email_client = Arc::new(MockEmailClient);
        let app_state = AppState::new(
            user_store,
            banned_store,
            two_fa_code_store.clone(),
            email_client,
        );

        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .unwrap();

        // Create new `TestApp` instance and return it
        Self {
            address,
            cookie_jar,
            http_client,
            two_fa_code_store,
            db_name,
            clean_up_called: false,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-2fa", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn clean_up(&mut self) {
        delete_database(&self.db_name).await;
        self.clean_up_called = true;
    }
}
impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.clean_up_called {
            panic!("TestApp::clean_up() was not called before dropping TestApp!");
        }
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

async fn configure_postgresql(db_name: &str) -> PgPool {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    configure_database(&postgresql_conn_url, &db_name).await;
    get_postgres_pool(&format!("{}/{}", postgresql_conn_url, db_name))
        .await
        .expect("Failed to create Postgres connection pool!")
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    let connection = PgPoolOptions::new().connect(db_conn_string).await.unwrap();
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .unwrap();
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);
    let connection = PgPoolOptions::new().connect(&db_conn_string).await.unwrap();
    sqlx::migrate!().run(&connection).await.unwrap();
}

fn configure_redis() -> RedisConnection {
    get_redis_client(DEFAULT_REDIS_HOSTNAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DATABASE_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}
