use crate::{NetworkError, Temperature};
use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct AppState {
    pub telemetry: Arc<RwLock<HashMap<String, Temperature>>>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/devices/{id}/temperature", get(get_temperature))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn get_temperature(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Temperature>, StatusCode> {
    state
        .telemetry
        .read()
        .await
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

pub async fn record(state: &AppState, value: Temperature) -> Result<(), NetworkError> {
    value.validate()?;
    state
        .telemetry
        .write()
        .await
        .insert(value.device_id.clone(), value);
    Ok(())
}
