use std::{env, io, net::SocketAddr, path::Path};

use leptos::config::{get_configuration, LeptosOptions};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub struct AppConfig {
    pub database_url: String,
    pub leptos_options: LeptosOptions,
    pub rust_log: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        load_dotenv_file(".env")?;
        load_dotenv_file(".env.production")?;

        let mut leptos_options = get_configuration(None)?.leptos_options;
        let database_url = required_env("DATABASE_URL")?;
        let site_addr: SocketAddr = required_env("LEPTOS_SITE_ADDR")?.parse().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("LEPTOS_SITE_ADDR must be a valid socket address: {error}"),
            )
        })?;
        let rust_log = required_env("RUST_LOG")?;

        leptos_options.site_addr = site_addr;

        Ok(Self {
            database_url,
            leptos_options,
            rust_log,
        })
    }

    pub fn init_tracing(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing_subscriber::registry()
            .with(EnvFilter::try_new(&self.rust_log)?)
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_current_span(true)
                    .with_span_list(true),
            )
            .try_init()?;

        Ok(())
    }
}

fn load_dotenv_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if Path::new(path).exists() {
        dotenvy::from_filename(path)?;
    }

    Ok(())
}

fn required_env(key: &'static str) -> Result<String, io::Error> {
    env::var(key).map_err(|error| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("{key} must be set: {error}"),
        )
    })
}
