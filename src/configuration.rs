use config::{Config, ConfigError, File};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::convert::TryFrom;
use std::io::Write;
use std::net::IpAddr;
use std::path::PathBuf;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Deserialize, Clone)]
pub struct AppBaseUrl(pub String);

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub logging: LoggingSettings,
}

#[derive(Deserialize, Clone)]
pub struct ApplicationSettings {
    pub base_url: AppBaseUrl,
    pub host: IpAddr,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: IpAddr,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host.to_string())
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum LoggingSettings {
    /// Ordering of the enums matters since serde tries to desirialize top to bottom
    File(FileLoggingSettings),
    Stdout(StdoutLoggingSettings),
}

impl MakeWriter<'_> for LoggingSettings {
    type Writer = CustomWriter;

    fn make_writer(&self) -> Self::Writer {
        match self {
            Self::Stdout(stdout_settings) => {
                if stdout_settings.enable {
                    CustomWriter::Stdout(std::io::stdout())
                } else {
                    CustomWriter::Empty(std::io::empty())
                }
            }
            Self::File(file_settings) => {
                if file_settings.enable {
                    CustomWriter::RotatingFile(tracing_appender::rolling::RollingFileAppender::new(
                        (&file_settings.rotation).into(),
                        &file_settings.dir,
                        &file_settings.file_prepend,
                    ))
                } else {
                    CustomWriter::Empty(std::io::empty())
                }
            }
        }
    }
}

pub enum CustomWriter {
    Empty(std::io::Empty),
    Stdout(std::io::Stdout),
    RotatingFile(tracing_appender::rolling::RollingFileAppender),
}

impl Write for CustomWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            CustomWriter::Stdout(writer) => writer.write(buf),
            CustomWriter::RotatingFile(writer) => writer.write(buf),
            CustomWriter::Empty(writer) => writer.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            CustomWriter::Stdout(writer) => writer.flush(),
            CustomWriter::RotatingFile(writer) => writer.flush(),
            CustomWriter::Empty(writer) => writer.flush(),
        }
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct StdoutLoggingSettings {
    pub enable: bool,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct FileLoggingSettings {
    pub enable: bool,
    pub dir: PathBuf,
    pub file_prepend: String,
    pub rotation: RotationWrapper,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RotationWrapper(String);

impl From<&RotationWrapper> for Rotation {
    fn from(val: &RotationWrapper) -> Self {
        match val.0.to_lowercase().as_ref() {
            "minutely" => Rotation::MINUTELY,
            "hourly" => Rotation::HOURLY,
            "daily" => Rotation::DAILY,
            "never" => Rotation::NEVER,
            _ => Rotation::DAILY,
        }
    }
}
impl From<RotationWrapper> for Rotation {
    fn from(val: RotationWrapper) -> Self {
        (&val).into()
    }
}

enum Environment {
    Local,
    Dev,
    Prod,
}

impl Environment {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Dev => "dev",
            Self::Prod => "prod",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "local" => Ok(Self::Local),
            "dev" => Ok(Self::Dev),
            "prod" => Ok(Self::Prod),
            err_env => Err(format!("no such Environment supported: {err_env}")),
        }
    }
}

pub fn load_settings() -> Result<Settings, ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    // Detect the running environment.
    // Default to `local` if unspecified.
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = Config::builder()
        .add_source(File::from(configuration_directory.join("base.yaml")))
        .add_source(File::from(
            configuration_directory.join(environment_filename),
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}
