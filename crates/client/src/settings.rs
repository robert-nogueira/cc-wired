use crate::fswatcher::WatchTarget;
use config::{Config, ConfigError};
use serde::Deserialize;

fn default_connect_timeout() -> u64 {
    10
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub ws_url: String,
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,
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
