use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use network_lab::{
    Temperature,
    api::{AppState, record, router},
};
use tower::ServiceExt;

#[tokio::test]
async fn records_and_reads_temperature_state() {
    let state = AppState::default();
    let value = Temperature {
        device_id: "d-1".into(),
        value: 22.0,
        unit: "C".into(),
        message_id: "m-1".into(),
    };
    record(&state, value.clone()).await.unwrap();
    assert_eq!(state.telemetry.read().await.get("d-1"), Some(&value));
}

#[tokio::test]
async fn health_route_is_public() {
    let app = router(AppState::default());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
