//! Immutable binary identity and consumer-facing contract metadata.
use crate::{config_error, error::ApiError, RuntimeProfile};

pub const VERSION: &str = env!("ARAF_BUILD_VERSION");
pub const SOURCE_SHA: &str = env!("ARAF_BUILD_SHA");
pub const RELEASE_BUILD: &str = env!("ARAF_BUILD_RELEASE");
pub const CONTRACT: &str = include_str!("../contracts/release-contract.json");

pub fn contracts() -> serde_json::Value {
    serde_json::from_str(CONTRACT).expect("CI-validated release contract")
}

pub fn validate(profile: RuntimeProfile) -> Result<(), ApiError> {
    if RELEASE_BUILD == "true" {
        if profile != RuntimeProfile::Production {
            return Err(config_error(
                "release binaries require the production profile",
            ));
        }
        for (name, built) in [("ARAF_VERSION", VERSION), ("ARAF_GIT_SHA", SOURCE_SHA)] {
            if std::env::var(name).is_ok_and(|value| value != built) {
                return Err(config_error(format!(
                    "{name} differs from immutable binary identity"
                )));
            }
        }
    }
    if let Ok(declared) = std::env::var("ARAF_O3K_API_CONTRACT") {
        if declared != contracts()["o3k"]["api_contract"].as_str().unwrap() {
            return Err(config_error("incompatible declared O3K API contract"));
        }
    }
    if let Ok(path) = std::env::var("SSL_CERT_FILE") {
        let pem = std::fs::read(path)
            .map_err(|_| config_error("SSL_CERT_FILE must be a readable PEM CA bundle"))?;
        if reqwest::Certificate::from_pem_bundle(&pem)
            .map_err(|_| config_error("SSL_CERT_FILE must contain a valid PEM CA bundle"))?
            .is_empty()
        {
            return Err(config_error(
                "SSL_CERT_FILE must contain a valid PEM CA bundle",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn contracts_are_schema_valid_and_do_not_claim_cloud_authority() {
        let contract = super::contracts();
        assert_eq!(contract["fixture_mode_allowed"], false);
        assert_eq!(contract["o3k"]["authority"], "O3K");
        assert_eq!(
            contract["o3k"]["stable_release_range"],
            serde_json::Value::Null
        );
        assert_eq!(contract["schema_version"], 1);
    }
}
