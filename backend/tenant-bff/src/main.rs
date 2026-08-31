//! Araf Tenant Console BFF.
//!
//! Serves only the Tenant Console surface; operator capability must never be
//! reachable through this process (ADR 0001).

use console_bff_core::{router, BffSurface};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("ARAF_TENANT_BFF_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let app = router(BffSurface {
        service: "tenant-bff",
    });

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    eprintln!("tenant-bff listening on {port}");
    axum::serve(listener, app).await
}
