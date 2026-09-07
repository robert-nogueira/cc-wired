use crate::fswatcher::WatchTarget;
use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub ws_url: String,
    pub watch: Vec<WatchTarget>,
}

impl Settings {
    pub fn load(config_file_name: Option<&str>) -> Result<Self, ConfigError> {
        let builder = Config::builder()
            .add_source(config::Environment::default())
            .add_source(
                config::File::with_name(config_file_name.unwrap_or("config"))
                    .format(config::FileFormat::Toml)
                    .required(true),
            );
        let settings: Settings = builder.build()?.try_deserialize()?;
        Ok(settings)
    }
}
