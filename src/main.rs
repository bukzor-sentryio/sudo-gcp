use clap::Parser;
use std::{os::unix::process::CommandExt, process::Command};

use sudo_gcp::{get_access_token, get_gcloud_config, Email, Lifetime, Scopes};

#[derive(Debug, Parser)]
#[command(author, version)]
struct Args {
    /// Email of service account to impersonate
    service_account: Email,
    /// Comma separated list of oauth scopes
    #[arg(short, long, default_value_t = Scopes::default())]
    scopes: Scopes,
    /// Add scopes in addition to the default
    #[arg(short, long)]
    append_scopes: Option<Scopes>,
    /// Lifetime of access token in seconds
    #[arg(long, default_value_t = Lifetime::default())]
    lifetime: Lifetime,
    /// Command to run with temporary elevated privileges
    command: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let args = Args::parse();

    let config = get_gcloud_config();
    let access_token =
        get_access_token(&config, &args.service_account, &args.lifetime, &args.scopes)?;

    let mut command_iter = args.command.iter();
    let command_exe = command_iter.next().unwrap();
    let command_args: Vec<String> = command_iter.map(|s| s.to_string()).collect();

    Err(Command::new(command_exe)
        .args(command_args)
        .env("GOOGLE_OAUTH_ACCESS_TOKEN", access_token.as_ref())
        .env("CLOUDSDK_AUTH_ACCESS_TOKEN", access_token.as_ref())
        .exec()
        .into())

    // TODO: use keyring (https://crates.io/crates/keyring) to cache the tokens
    //        and check the timestamps before getting another access token
}
