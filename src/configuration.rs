use bb8_redis::redis;
use config::{Config, ConfigError, File};
use redis::ProtocolVersion;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::convert::TryFrom;
use std::io::Write;
use std::net::IpAddr;
use std::path::PathBuf;
use tracing_appender::rolling::Rotation;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub logging: LoggingSettings,
}

#[derive(Deserialize, Clone)]
pub struct ApplicationSettings {
    pub base_url: String,
    pub host: IpAddr,
    pub port: u16,
    pub hmac_secret: SecretString,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
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
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct RedisSettings {
    pub username: Option<String>,
    pub password: Option<SecretString>,
    pub port: u16,
    pub host: String,
    #[serde(deserialize_with = "protocol_from_string")]
    pub protocol: ProtocolVersion,
    pub database_number: Option<i64>,
}

fn protocol_from_string<'de, D>(deserializer: D) -> Result<ProtocolVersion, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: std::borrow::Cow<String> = Deserialize::deserialize(deserializer)?;
    match s.as_str() {
        "2" | "v2" | "resp2" => Ok(ProtocolVersion::RESP2),
        "3" | "v3" | "resp3" => Ok(ProtocolVersion::RESP3),
        default => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Str(default),
            &r#""v3" or "v2" redis protocol version"#,
        )),
    }
}

impl RedisSettings {
    pub fn with_db(&self) -> redis::ConnectionInfo {
        redis::ConnectionInfo {
            addr: redis::ConnectionAddr::Tcp(self.host.clone(), self.port),
            redis: redis::RedisConnectionInfo {
                db: self.database_number.unwrap_or(0),
                protocol: self.protocol,
                username: self.username.clone(),
                password: self.password.as_ref().map(|x| x.expose_secret().to_owned()),
            },
        }
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(tag = "dest", rename_all = "lowercase")]
pub enum LoggingSettings {
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
                        file_settings.rotation.clone(),
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
    #[serde(deserialize_with = "rotation_from_string")]
    pub rotation: Rotation,
}

fn rotation_from_string<'de, D>(deserializer: D) -> Result<Rotation, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: std::borrow::Cow<String> = Deserialize::deserialize(deserializer)?;
    match s.as_str() {
        "minutely" => Ok(Rotation::MINUTELY),
        "hourly" => Ok(Rotation::HOURLY),
        "daily" => Ok(Rotation::DAILY),
        "never" => Ok(Rotation::NEVER),
        default => Err(serde::de::Error::invalid_value(
            serde::de::Unexpected::Str(default),
            &r#""daily" or "hourly" or "daily" or "never""#,
        )),
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
    let environment: Environment = std::env::var("APP__ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP__ENVIRONMENT.");
    let environment_filename = format!("{}.yaml", environment.as_str());
    let settings = Config::builder()
        .add_source(File::from(configuration_directory.join("base.yaml")))
        .add_source(File::from(
            configuration_directory.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP__APPLICATION_PORT=5001 would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("__")
                .separator("_"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}
