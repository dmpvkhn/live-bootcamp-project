use std::error::Error;
pub mod model;
use axum::http::StatusCode;
use axum::{
    response::{Html, IntoResponse},
    routing::{get, post},
    serve::Serve,
    Router,
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
pub mod routes;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<TcpListener, Router, Router>,
    pub address: String,
}

impl Application {
    pub async fn build(address: &str) -> Result<Self, Box<dyn Error>> {
        let assets_dir = ServeDir::new("assets");
        let router = Router::new()
            .fallback_service(assets_dir)
            .route("/hello", get(hello_handler))
            .route("/signup", post(routes::signup))
            .route("/login", post(routes::login))
            .route("/verify-2fa", post(routes::verify_2fa))
            .route("/logout", post(routes::logout))
            .route("/verify-token", post(routes::verify_token));

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        Ok(Self { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        println!("listening on {}", &self.address);
        self.server.await
    }
}

async fn hello_handler() -> Html<&'static str> {
    // TODO: Update this to a custom message!
    Html("<h1>Hello, World!</h1>")
}
