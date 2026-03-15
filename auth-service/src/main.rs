use auth_service::{
    get_postgres_pool,
    services::{
        mock_email_client::MockEmailClient, HashmapBannedTokenStore, HashmapTwoFACodeStore,
        HashmapUserStore, PostgresUserStore, RedisBannedTokenStore, RedisTwoFACodeStore,
    },
    utils::{
        constants::{prod, DATABASE_URL},
        tracing::init_tracing,
    },
    AppState, Application,
};
use auth_service::{get_redis_client, utils::constants::REDIS_HOST_NAME};
use redis::Connection;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
#[tokio::main]
async fn main() {
    init_tracing();

    let pg_pool = configure_postgresql().await;
    let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
    let redis_conn = Arc::new(RwLock::new(configure_redis()));
    let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(redis_conn.clone())));
    let twofa_token_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(redis_conn)));
    let email_client = Arc::new(MockEmailClient);
    let app_state = AppState::new(
        user_store,
        banned_token_store,
        twofa_token_store,
        email_client,
    );

    let pg_pool = configure_postgresql().await;

    let app = Application::build(app_state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build app");

    app.run().await.expect("Failed to run app");
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

fn configure_redis() -> Connection {
    get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}
