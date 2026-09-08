use std::collections::HashMap;
use std::sync::Mutex;

/// Shared table of connected consumer sessions, keyed by computer id.
#[derive(Default)]
pub struct Registry(Mutex<HashMap<String, actix_ws::Session>>);

pub enum RouteOutcome {
    Delivered,
    NotConnected,
    SendFailed,
}

impl Registry {
    pub fn register(&self, computer_id: String, session: actix_ws::Session) {
        self.0.lock().unwrap().insert(computer_id, session);
    }

    pub fn unregister(&self, computer_id: &str) {
        self.0.lock().unwrap().remove(computer_id);
    }

    fn get(&self, computer_id: &str) -> Option<actix_ws::Session> {
        self.0.lock().unwrap().get(computer_id).cloned()
    }

    /// Forwards `text` verbatim to the session registered under `computer_id`.
    /// Removes the session if the send fails (mirrors prior behavior).
    pub async fn route(&self, computer_id: &str, text: &str) -> RouteOutcome {
        let Some(mut session) = self.get(computer_id) else {
            return RouteOutcome::NotConnected;
        };
        if session.text(text.to_owned()).await.is_err() {
            self.unregister(computer_id);
            return RouteOutcome::SendFailed;
        }
        RouteOutcome::Delivered
    }
}
