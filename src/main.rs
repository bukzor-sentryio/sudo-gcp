use std::{
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
};

use clap::Parser;
use config::{Config, Environment, File, FileFormat};
use serde::Deserialize;
use sudo_gcp::{get_access_token, get_gcloud_config, Delegates, Email, Lifetime, Scopes};

#[derive(Debug, Deserialize)]
struct Settings {
    service_account: Email,
    delegates: Option<Delegates>,
    #[serde(default)]
    scopes: Scopes,
    #[serde(default)]
    lifetime: Lifetime,
}

#[derive(Debug, Parser)]
#[command(author, version)]
struct Args {
    /// Path to config file
    #[arg(short, long, default_value = "./sudo-gcp.toml")]
    config_file: PathBuf,
    // /// Email of service account to impersonate
    // #[arg(short, long)]
    // service_account: Option<Email>,
    // /// Comma separated list of oauth scopes
    // #[arg(long, default_value_t = Scopes::default())]
    // scopes: Scopes,
    // /// Add scopes in addition to the default
    // #[arg(short, long)]
    // append_scopes: Option<Scopes>,
    // /// Lifetime of access token in seconds
    // #[arg(long, default_value_t = Lifetime::default())]
    // lifetime: Lifetime,
    /// Command to run with temporary elevated privileges
    command: Vec<String>,
}

fn get_settings<P: AsRef<Path>>(path: P) -> Result<Settings, config::ConfigError> {
    let settings_file_path = path.as_ref().to_str().unwrap();
    let settings = Config::builder()
        .add_source(File::new(settings_file_path, FileFormat::Toml))
        .add_source(Environment::with_prefix("sudogcp"))
        .build()?;
    settings.try_deserialize::<Settings>()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();
    let settings = get_settings(args.config_file)?;

    let config = get_gcloud_config()?;

    let access_token = get_access_token(
        &config,
        settings.delegates,
        &settings.service_account,
        &settings.lifetime,
        &settings.scopes,
    )?;

    let mut command_iter = args.command.iter();
    let command_exe = command_iter.next().unwrap();
    let command_args: Vec<String> = command_iter.map(|s| s.to_string()).collect();

    Err(Command::new(command_exe)
        .args(command_args)
        .env("GOOGLE_OAUTH_ACCESS_TOKEN", access_token.as_ref())
        .env("CLOUDSDK_AUTH_ACCESS_TOKEN", access_token.as_ref())
        .exec()
        .into())
}
