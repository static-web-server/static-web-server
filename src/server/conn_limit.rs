// SPDX-License-Identifier: MIT OR Apache-2.0
// This file is part of Static Web Server.
// See https://static-web-server.net/ for more information
// Copyright (C) 2019-present Jose Quintana <joseluisq.net>

//! Limit on the number of connections served at the same time, shared by all listeners.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::Result;

/// Minimum time between two warnings that the limit is reached,
/// so a server that stays at its limit doesn't log a line for every closed connection.
const WARNING_INTERVAL: Duration = Duration::from_secs(60);

/// Shared limit on the number of connections served at the same time.
///
/// Each accept loop takes a slot for the connection it just accepted and doesn't accept again
/// until it has one, so once the limit is reached new connections wait in the listen backlog.
#[derive(Clone, Debug)]
pub(crate) struct ConnLimit {
    /// One permit per connection slot. It's never closed.
    slots: Arc<Semaphore>,
    /// Maximum number of connections served at the same time.
    max: usize,
    /// Time of the last "limit reached" warning.
    last_warning: Arc<Mutex<Option<Instant>>>,
}

impl ConnLimit {
    /// Create a limit of `max` connections served at the same time, or `None` for no limit when `max` is `0`.
    pub(crate) fn new(max: usize) -> Result<Option<Self>> {
        if max == 0 {
            return Ok(None);
        }
        if max > Semaphore::MAX_PERMITS {
            bail!(
                "max connections must be at most {}, got {max}",
                Semaphore::MAX_PERMITS
            );
        }
        Ok(Some(Self {
            slots: Arc::new(Semaphore::new(max)),
            max,
            last_warning: Arc::default(),
        }))
    }

    /// Maximum number of connections served at the same time.
    #[cfg(test)]
    pub(crate) fn max(&self) -> usize {
        self.max
    }

    /// Wait for a free connection slot, which is released when the returned permit is dropped.
    pub(crate) async fn acquire(&self) -> OwnedSemaphorePermit {
        if let Ok(permit) = self.slots.clone().try_acquire_owned() {
            return permit;
        }
        self.warn_limit_reached();
        let Ok(permit) = self.slots.clone().acquire_owned().await else {
            unreachable!("the connection limit semaphore is never closed");
        };
        permit
    }

    /// Log that the limit is reached, at most once per [`WARNING_INTERVAL`].
    #[cold]
    fn warn_limit_reached(&self) {
        let Ok(mut last_warning) = self.last_warning.lock() else {
            return;
        };
        let now = Instant::now();
        if last_warning.is_some_and(|last| now.duration_since(last) < WARNING_INTERVAL) {
            return;
        }
        *last_warning = Some(now);
        tracing::warn!(
            max_connections = self.max,
            "connection limit reached, new connections wait until others close"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::ConnLimit;
    use std::time::Duration;
    use tokio::sync::Semaphore;

    #[test]
    fn conn_limit_zero_means_no_limit() {
        assert!(ConnLimit::new(0).unwrap().is_none());
    }

    #[test]
    fn conn_limit_rejects_values_above_semaphore_max() {
        let err = ConnLimit::new(Semaphore::MAX_PERMITS + 1).unwrap_err();
        assert!(err.to_string().contains("max connections must be at most"));
        assert!(ConnLimit::new(Semaphore::MAX_PERMITS).unwrap().is_some());
    }

    #[tokio::test]
    async fn conn_limit_waits_until_a_slot_is_released() {
        let limit = ConnLimit::new(2).unwrap().unwrap();
        let first = limit.acquire().await;
        let _second = limit.acquire().await;

        let third = limit.acquire();
        tokio::pin!(third);
        assert!(
            tokio::time::timeout(Duration::from_millis(100), &mut third)
                .await
                .is_err(),
            "a third slot was granted with a limit of 2"
        );

        drop(first);
        let _third = tokio::time::timeout(Duration::from_secs(5), third)
            .await
            .expect("slot not granted after one was released");
    }

    #[tokio::test]
    async fn conn_limit_clones_share_slots() {
        let limit = ConnLimit::new(1).unwrap().unwrap();
        let other = limit.clone();
        let _permit = limit.acquire().await;
        assert!(
            tokio::time::timeout(Duration::from_millis(100), other.acquire())
                .await
                .is_err(),
            "a clone granted a slot past the shared limit"
        );
    }

    #[tokio::test]
    async fn conn_limit_warns_once_per_interval() {
        let (logs, _guard) = crate::server::tests::capture_debug_logs();
        let limit = ConnLimit::new(1).unwrap().unwrap();
        let permit = limit.acquire().await;
        // Hit the limit twice in a row: only the first wait is logged.
        for _ in 0..2 {
            let _ = tokio::time::timeout(Duration::from_millis(50), limit.acquire()).await;
        }
        drop(permit);

        let lines = logs.lines("connection limit reached");
        assert_eq!(lines.len(), 1, "expected one warning, got: {lines:?}");
        assert!(lines[0].contains("WARN"), "{}", lines[0]);
        assert!(lines[0].contains("max_connections=1"), "{}", lines[0]);
    }
}
