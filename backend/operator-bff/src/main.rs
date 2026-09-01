//! Araf Operator Console BFF.
//!
//! Serves only the Operator Console surface, deployable on a management
//! network with its own session/trust boundary (ADR 0001).

use console_bff_core::fixture_router;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let port = std::env::var("ARAF_OPERATOR_BFF_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8081);

    let app = fixture_router("operator-bff");

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(%port, "operator-bff listening");
    axum::serve(listener, app).await
}
