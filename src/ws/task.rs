//! Helpers for owning spawned [`tokio`] tasks.
//!
//! The reconnection handlers in [`crate::clob::ws::subscription`] and
//! [`crate::rtds::subscription`] spawn detached tasks that each hold a
//! strong `Arc<SubscriptionManager>` clone. Those tasks also own a clone
//! of the underlying [`crate::ws::ConnectionManager`], whose `watch::Sender`
//! is what the task awaits on for state changes. Because the `Sender` only
//! closes when every clone of it drops, and a strong `Arc` clone inside the
//! spawned task prevents the owning `SubscriptionManager` (and therefore
//! that `ConnectionManager` clone) from dropping, the task can never exit
//! on its own — a reference cycle that leaks the entire channel for the
//! lifetime of the process (issue #325).
//!
//! [`AbortOnDrop`] is a thin wrapper around [`tokio::task::JoinHandle`]
//! that calls `abort()` on drop. Clients store the wrapped handle next to
//! the `Arc<SubscriptionManager>`; when the client (and therefore the
//! wrapper) drops, the handler task is aborted, its stack locals — which
//! include the strong `Arc` clone — are released, and the whole graph can
//! drop normally.

use tokio::task::JoinHandle;

/// Owns a [`JoinHandle`] and calls [`JoinHandle::abort`] on drop.
///
/// See the module-level documentation for the cycle this breaks.
pub(crate) struct AbortOnDrop(JoinHandle<()>);

impl AbortOnDrop {
    /// Wrap a spawned task handle so it is aborted when this value drops.
    #[must_use]
    pub(crate) fn new(handle: JoinHandle<()>) -> Self {
        Self(handle)
    }
}

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        // `JoinHandle::abort` is a no-op if the task has already completed,
        // so this is always safe to call.
        self.0.abort();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use super::AbortOnDrop;

    #[tokio::test]
    async fn abort_on_drop_cancels_pending_task() {
        let finished = Arc::new(AtomicBool::new(false));
        let finished_task = Arc::clone(&finished);

        let handle = tokio::spawn(async move {
            // Park forever unless aborted.
            std::future::pending::<()>().await;
            finished_task.store(true, Ordering::SeqCst);
        });

        let wrapper = AbortOnDrop::new(handle);
        drop(wrapper);

        // Give the runtime a moment to process the abort.
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(
            !finished.load(Ordering::SeqCst),
            "task body should never have run to completion after abort"
        );
    }

    #[tokio::test]
    async fn abort_on_drop_is_noop_for_finished_task() {
        // Spawn a task that completes immediately, wait for it, then drop
        // the wrapper. `JoinHandle::abort` on a finished task is documented
        // as a no-op, so this must not panic or error.
        let wrapper = AbortOnDrop::new(tokio::spawn(async {}));
        tokio::time::sleep(Duration::from_millis(10)).await;
        drop(wrapper);
    }
}
