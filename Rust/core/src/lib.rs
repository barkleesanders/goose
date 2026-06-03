#![recursion_limit = "256"]
// Clippy: these are structural/cosmetic lints, not bugs. Refactoring the
// remaining multi-argument functions into parameter structs, boxing large
// `Err` variants, factoring complex return tuples into named types, rewriting
// large literal row lists as `vec![]`, and converting index loops are tracked
// follow-ups (same class as file-splitting / perf work), not alpha ship
// blockers. Re-enable individually when doing that refactor pass.
#![allow(
    clippy::too_many_arguments,
    clippy::result_large_err,
    clippy::type_complexity,
    clippy::vec_init_then_push,
    clippy::needless_range_loop,
    clippy::redundant_guards
)]

pub mod activity_candidates;
pub mod activity_identity;
pub mod activity_sessions;
pub mod algorithm_compare;
pub mod bridge;
pub mod calibration;
pub mod capture_correlation;
pub mod capture_import;
pub mod capture_sanitize;
pub mod commands;
pub mod debug_ws;
pub mod debug_ws_server;
pub mod energy_rollup;
mod error;
pub mod export;
pub mod fixtures;
pub mod health_sync;
pub mod historical_sync;
pub mod local_health_validation;
pub mod metric_features;
pub mod metric_readiness;
pub mod metrics;
pub mod openwhoop_reference;
pub mod perf_budget;
pub mod privacy_lint;
pub mod property_tests;
pub mod protocol;
pub mod recovery_rollup;
pub mod reference;
pub mod report;
pub mod sleep_validation;
pub mod step_counter;
pub mod step_discovery;
pub mod step_motion_estimator;
pub mod storage_check;
pub mod store;
pub mod timeline;
pub mod tool_args;
pub mod ui_coverage;
pub mod validation_labels;

pub use error::{GooseError, GooseResult};
