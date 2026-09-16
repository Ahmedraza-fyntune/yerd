//! Error type for tunnel supervision.

use yerd_supervise::ExitReason;

/// Failures from supervising a `cloudflared` tunnel.
///
/// Pure validation never fails today (origin/arg/config generation is total), so
/// these are all runtime failures from the async manager (the I/O edge).
#[derive(Debug, thiserror::Error)]
pub enum TunnelError {
    /// The tunnel binary has not been installed yet.
    #[error("tunnel binary is not installed: {0}")]
    NotInstalled(String),

    /// Spawning the tunnel process failed.
    #[error("failed to spawn tunnel process: {0}")]
    Spawn(#[source] std::io::Error),

    /// Waiting on / signalling the child failed.
    #[error("tunnel process I/O failed: {0}")]
    Io(#[source] std::io::Error),

    /// The readiness window elapsed before the tunnel came up (no URL / no edge
    /// registration line appeared).
    #[error("tunnel for {site} did not become ready in time")]
    ReadinessTimedOut {
        /// The site whose tunnel timed out.
        site: String,
    },

    /// The tunnel exited repeatedly and exhausted its restart budget.
    #[error("tunnel for {site} failed permanently (last exit: {last_exit})")]
    PermanentFailure {
        /// The site whose tunnel failed.
        site: String,
        /// Exit reason of the final attempt.
        last_exit: ExitReason,
    },
}
