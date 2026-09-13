//! Bounded, process-local telemetry for the BFF.
//!
//! Metrics intentionally use a fixed set of labels.  Request, resource,
//! tenant and operation identifiers are useful in logs/traces, but must never
//! become unbounded Prometheus labels.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

const SURFACES: [&str; 2] = ["tenant-bff", "operator-bff"];
const BACKENDS: [&str; 2] = ["o3k", "openstack"];
const LATENCY_BUCKETS_MS: [u64; 7] = [5, 25, 100, 250, 500, 1_000, 5_000];

#[derive(Debug)]
struct Counters {
    requests: [[AtomicU64; 6]; 2],
    latency_buckets: [[AtomicU64; 7]; 2],
    latency_sum_ms: [AtomicU64; 2],
    latency_count: [AtomicU64; 2],
    backend_calls: [[AtomicU64; 6]; 2],
    auth_failures: [AtomicU64; 2],
    rate_limited: [AtomicU64; 2],
    compatibility_reconciliations: [AtomicU64; 3],
}

impl Default for Counters {
    fn default() -> Self {
        Self {
            requests: std::array::from_fn(|_| std::array::from_fn(|_| AtomicU64::new(0))),
            latency_buckets: std::array::from_fn(|_| std::array::from_fn(|_| AtomicU64::new(0))),
            latency_sum_ms: std::array::from_fn(|_| AtomicU64::new(0)),
            latency_count: std::array::from_fn(|_| AtomicU64::new(0)),
            backend_calls: std::array::from_fn(|_| std::array::from_fn(|_| AtomicU64::new(0))),
            auth_failures: std::array::from_fn(|_| AtomicU64::new(0)),
            rate_limited: std::array::from_fn(|_| AtomicU64::new(0)),
            compatibility_reconciliations: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }
}

static COUNTERS: OnceLock<Counters> = OnceLock::new();

fn counters() -> &'static Counters {
    COUNTERS.get_or_init(Counters::default)
}

fn surface_index(surface: &str) -> usize {
    SURFACES
        .iter()
        .position(|item| *item == surface)
        .unwrap_or(0)
}

fn backend_index(backend: &str) -> usize {
    BACKENDS
        .iter()
        .position(|item| *item == backend)
        .unwrap_or(0)
}

fn status_index(status: u16) -> usize {
    match status {
        200..=299 => 0,
        300..=399 => 1,
        400..=499 => 2,
        500..=599 => 3,
        _ => 4,
    }
}

pub fn record_request(surface: &str, status: u16, elapsed_ms: u64) {
    let index = surface_index(surface);
    let c = counters();
    c.requests[index][status_index(status)].fetch_add(1, Ordering::Relaxed);
    c.latency_sum_ms[index].fetch_add(elapsed_ms, Ordering::Relaxed);
    c.latency_count[index].fetch_add(1, Ordering::Relaxed);
    let bucket = LATENCY_BUCKETS_MS
        .iter()
        .position(|limit| elapsed_ms <= *limit)
        .unwrap_or(LATENCY_BUCKETS_MS.len() - 1);
    c.latency_buckets[index][bucket].fetch_add(1, Ordering::Relaxed);
}

pub fn record_backend_call(backend: &str, status: u16) {
    let c = counters();
    c.backend_calls[backend_index(backend)][status_index(status)].fetch_add(1, Ordering::Relaxed);
}

pub fn record_auth_failure(surface: &str) {
    counters().auth_failures[surface_index(surface)].fetch_add(1, Ordering::Relaxed);
}

pub fn record_rate_limited(surface: &str) {
    counters().rate_limited[surface_index(surface)].fetch_add(1, Ordering::Relaxed);
}

/// Record a bounded reconciliation outcome: `succeeded`, `failed`, or
/// `skipped`.
pub fn record_compatibility_reconciliation(outcome: &str) {
    let index = match outcome {
        "succeeded" => 0,
        "failed" => 1,
        _ => 2,
    };
    counters().compatibility_reconciliations[index].fetch_add(1, Ordering::Relaxed);
}

/// Render Prometheus text exposition.  No user/resource/request values are
/// included in labels, keeping cardinality bounded by the fixed dimensions.
pub fn render_prometheus() -> String {
    let c = counters();
    let mut output = String::from(
        "# HELP araf_bff_requests_total HTTP requests handled by the BFF.\n# TYPE araf_bff_requests_total counter\n",
    );
    for (surface_index, surface) in SURFACES.iter().enumerate() {
        for (class, label) in ["2xx", "3xx", "4xx", "5xx", "other"].iter().enumerate() {
            output.push_str(&format!(
                "araf_bff_requests_total{{surface=\"{surface}\",status_class=\"{label}\"}} {}\n",
                c.requests[surface_index][class].load(Ordering::Relaxed)
            ));
        }
        output.push_str(&format!(
            "araf_bff_request_duration_ms_sum{{surface=\"{surface}\"}} {}\n",
            c.latency_sum_ms[surface_index].load(Ordering::Relaxed)
        ));
        output.push_str(&format!(
            "araf_bff_request_duration_ms_count{{surface=\"{surface}\"}} {}\n",
            c.latency_count[surface_index].load(Ordering::Relaxed)
        ));
        for (bucket_index, bound) in LATENCY_BUCKETS_MS.iter().enumerate() {
            output.push_str(&format!(
                "araf_bff_request_duration_ms_bucket{{surface=\"{surface}\",le=\"{bound}\"}} {}\n",
                c.latency_buckets[surface_index][bucket_index].load(Ordering::Relaxed)
            ));
        }
        output.push_str(&format!(
            "araf_bff_auth_failures_total{{surface=\"{surface}\"}} {}\n",
            c.auth_failures[surface_index].load(Ordering::Relaxed)
        ));
        output.push_str(&format!(
            "araf_bff_rate_limited_total{{surface=\"{surface}\"}} {}\n",
            c.rate_limited[surface_index].load(Ordering::Relaxed)
        ));
    }
    output.push_str(
        "# HELP araf_bff_backend_calls_total Upstream calls by backend and status class.\n# TYPE araf_bff_backend_calls_total counter\n",
    );
    for (backend_index, backend) in BACKENDS.iter().enumerate() {
        for (class, label) in ["2xx", "3xx", "4xx", "5xx", "other"].iter().enumerate() {
            output.push_str(&format!(
                "araf_bff_backend_calls_total{{backend=\"{backend}\",status_class=\"{label}\"}} {}\n",
                c.backend_calls[backend_index][class].load(Ordering::Relaxed)
            ));
        }
    }
    output.push_str(
        "# HELP araf_compatibility_reconciliations_total OpenStack compatibility reconciliation outcomes.\n# TYPE araf_compatibility_reconciliations_total counter\n",
    );
    for (index, outcome) in ["succeeded", "failed", "skipped"].iter().enumerate() {
        output.push_str(&format!(
            "araf_compatibility_reconciliations_total{{outcome=\"{outcome}\"}} {}\n",
            c.compatibility_reconciliations[index].load(Ordering::Relaxed)
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_use_only_bounded_labels() {
        record_request("tenant-bff", 500, 12);
        let output = render_prometheus();
        assert!(output.contains("surface=\"tenant-bff\""));
        assert!(!output.contains("resource-"));
        assert!(!output.contains("request_id"));
    }
}
