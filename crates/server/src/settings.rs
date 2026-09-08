use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub host: String,
    pub port: u16,
}

impl Settings {
    pub fn load(config_file_name: Option<&str>) -> Result<Self, ConfigError> {
        let builder = Config::builder()
            .set_default("host", "127.0.0.1")?
            .set_default("port", 8080)?
            .add_source(
                config::File::with_name(config_file_name.unwrap_or("config"))
                    .format(config::FileFormat::Toml)
                    .required(false),
            )
            .add_source(config::Environment::default());

        builder.build()?.try_deserialize()
    }
}
