//! Araf Operator Console BFF.
//!
//! Serves only the Operator Console surface, deployable on a management
//! network with its own session/trust boundary (ADR 0001).

use console_bff_core::{router, BffSurface};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("ARAF_OPERATOR_BFF_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8081);

    let app = router(BffSurface {
        service: "operator-bff",
    });

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
    eprintln!("operator-bff listening on {port}");
    axum::serve(listener, app).await
}
