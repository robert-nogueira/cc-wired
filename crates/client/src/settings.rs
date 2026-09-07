use crate::fswatcher::WatchTarget;
use config::{Config, ConfigError};
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub ws_url: String,
    pub watch: Vec<WatchTarget>,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let builder = Config::builder()
            .add_source(config::Environment::default())
            .add_source(
                config::File::with_name("config")
                    .format(config::FileFormat::Toml)
                    .required(true),
            );
        let settings: Settings = builder.build()?.try_deserialize()?;
        Ok(settings)
    }
}

pub static SETTINGS: LazyLock<Settings> =
    LazyLock::new(|| Settings::load().expect("Invalid config"));
