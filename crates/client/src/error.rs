use std::{error::Error as StdError, fmt};

use crate::ws;

#[derive(Debug)]
pub enum Error {
    Ws(ws::WsError),
    FsWatcher(notify::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ws(err) => write!(f, "WebSocket error: {err}"),
            Self::FsWatcher(err) => {
                write!(f, "filesystem watcher error: {err}")
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Ws(err) => Some(err),
            Self::FsWatcher(err) => Some(err),
        }
    }
}

impl From<ws::WsError> for Error {
    fn from(err: ws::WsError) -> Self {
        Self::Ws(err)
    }
}

impl From<notify::Error> for Error {
    fn from(err: notify::Error) -> Self {
        Self::FsWatcher(err)
    }
}
