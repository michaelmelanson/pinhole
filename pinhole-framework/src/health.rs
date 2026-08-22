use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// Shared server health/readiness state, checked by the `HealthCheck`
/// protocol message.
///
/// Starts healthy; flipped to draining once the server has stopped
/// accepting new connections during graceful shutdown, so a client polling
/// on an already-open connection can learn to stop routing new traffic
/// before the process actually exits.
#[derive(Clone)]
pub struct ServerHealth(Arc<AtomicBool>);

impl ServerHealth {
    pub fn new() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }

    pub fn is_healthy(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    /// Mark the server as no longer accepting new work. Called once, when
    /// graceful shutdown begins.
    pub fn mark_draining(&self) {
        self.0.store(false, Ordering::Relaxed);
    }
}

impl Default for ServerHealth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starts_healthy() {
        let health = ServerHealth::new();
        assert!(health.is_healthy());
    }

    #[test]
    fn test_mark_draining_flips_to_unhealthy() {
        let health = ServerHealth::new();
        health.mark_draining();
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_shared_via_clone() {
        let health = ServerHealth::new();
        let cloned = health.clone();

        cloned.mark_draining();

        assert!(!health.is_healthy(), "clones must share the same state");
    }
}
