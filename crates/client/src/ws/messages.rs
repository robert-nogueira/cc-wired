#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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
