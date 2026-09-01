//! Araf Tenant Console BFF.
//!
//! Serves only the Tenant Console surface; operator capability must never be
//! reachable through this process (ADR 0001).

use console_bff_core::{api_router_for_config, BffConfig};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let port = std::env::var("ARAF_TENANT_BFF_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let config = BffConfig::from_env("tenant-bff")
        .map_err(|e| std::io::Error::other(format!("invalid production configuration: {e}")))?;
    let app = api_router_for_config(config)
        .map_err(|e| std::io::Error::other(format!("failed to build router: {e}")))?;

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    tracing::info!(%port, "tenant-bff listening");
    axum::serve(listener, app).await
}
