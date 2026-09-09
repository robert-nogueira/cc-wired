use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    FileCreated {
        computer_id: String,
        path: String,
        contents: String,
    },
    FileChanged {
        computer_id: String,
        path: String,
        contents: String,
    },
    FileDeleted {
        computer_id: String,
        path: String,
    },
}

impl ClientMessage {
    pub fn computer_id(&self) -> &str {
        match self {
            ClientMessage::FileCreated { computer_id, .. }
            | ClientMessage::FileChanged { computer_id, .. }
            | ClientMessage::FileDeleted { computer_id, .. } => computer_id,
        }
    }
}
