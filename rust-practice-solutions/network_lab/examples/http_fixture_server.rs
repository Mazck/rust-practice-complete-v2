use axum::{
    Router,
    extract::Json,
    routing::{get, post},
};
use serde_json::{Value, json};

async fn get_json() -> Json<Value> {
    Json(json!({"url": "http://127.0.0.1:3010/get"}))
}

async fn post_json(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({"json": body}))
}

async fn status_ok() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/get", get(get_json))
        .route("/post", post(post_json))
        .route("/status/200", get(status_ok));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3010").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
