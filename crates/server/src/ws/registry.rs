use std::collections::HashMap;
use std::sync::Mutex;

/// Abstracts sending a text frame to a connected session, so `Registry` can
/// be unit-tested against a fake session instead of a real WS connection.
/// Send isn't required: every caller runs on actix-web's non-Send LocalSet.
#[allow(async_fn_in_trait)]
pub trait SessionSink {
    async fn send_text(&mut self, text: String) -> Result<(), SendError>;
}

/// Opaque error returned when the session can no longer be written to.
#[derive(Debug)]
pub struct SendError;

impl SessionSink for actix_ws::Session {
    async fn send_text(&mut self, text: String) -> Result<(), SendError> {
        self.text(text).await.map_err(|_| SendError)
    }
}

/// Shared table of connected consumer sessions, keyed by computer id.
pub struct Registry<S = actix_ws::Session>(Mutex<HashMap<String, S>>);

impl<S> Default for Registry<S> {
    fn default() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

#[derive(Debug)]
pub enum RouteOutcome {
    Delivered,
    NotConnected,
    SendFailed,
}

impl<S: SessionSink + Clone> Registry<S> {
    pub fn register(&self, computer_id: String, session: S) {
        self.0.lock().unwrap().insert(computer_id, session);
    }

    pub fn unregister(&self, computer_id: &str) {
        self.0.lock().unwrap().remove(computer_id);
    }

    fn get(&self, computer_id: &str) -> Option<S> {
        self.0.lock().unwrap().get(computer_id).cloned()
    }

    /// Forwards `text` verbatim to the session registered under `computer_id`.
    /// Removes the session if the send fails (mirrors prior behavior).
    pub async fn route(&self, computer_id: &str, text: &str) -> RouteOutcome {
        let Some(mut session) = self.get(computer_id) else {
            return RouteOutcome::NotConnected;
        };
        if session.send_text(text.to_owned()).await.is_err() {
            self.unregister(computer_id);
            return RouteOutcome::SendFailed;
        }
        RouteOutcome::Delivered
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default)]
    struct MockSession {
        should_fail: bool,
    }

    impl SessionSink for MockSession {
        async fn send_text(&mut self, _text: String) -> Result<(), SendError> {
            if self.should_fail {
                Err(SendError)
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn send_failure_unregisters_and_reports_send_failed() {
        let registry = Registry::<MockSession>::default();
        registry.register(
            "computer-1".to_string(),
            MockSession { should_fail: true },
        );

        let outcome = registry.route("computer-1", "hello").await;
        assert!(matches!(outcome, RouteOutcome::SendFailed));

        let outcome = registry.route("computer-1", "hello").await;
        assert!(matches!(outcome, RouteOutcome::NotConnected));
    }

    #[tokio::test]
    async fn send_success_reports_delivered() {
        let registry = Registry::<MockSession>::default();
        registry.register(
            "computer-1".to_string(),
            MockSession { should_fail: false },
        );

        let outcome = registry.route("computer-1", "hello").await;
        assert!(matches!(outcome, RouteOutcome::Delivered));
    }
}
