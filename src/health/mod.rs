use axum::{extract::State, http::StatusCode, routing::get, Router};
use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::PgPool;

pub async fn run(
    pool: PgPool,
    metrics_handler: PrometheusHandle,
    port: u16,
) -> std::io::Result<()> {
    let app = Router::new()
        .route(
            "/metrics",
            get(move || async move { metrics_handler.render() }),
        )
        .route("/health", get(health))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    log::info!("health server is stared on port: {}", port);
    axum::serve(listener, app).await
}

async fn health(State(pool): State<PgPool>) -> StatusCode {
    let request = sqlx::query("SELECT 1").execute(&pool);
    match request.await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            log::error!("db connection error: {}", err);
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}
