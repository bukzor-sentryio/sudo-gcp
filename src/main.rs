use clap::Parser;

use sudo_gcp::{get_access_token, get_gcloud_config, Email, Lifetime, Scopes};

#[derive(Debug, Parser)]
#[command(author, version)]
struct Args {
    /// Email of service account to impersonate
    service_account: Email,
    /// Comma separated list of oauth scopes
    #[arg(short, long, default_value_t = Scopes::default())]
    scopes: Scopes,
    /// Lifetime of access token in seconds
    #[arg(long, default_value_t = Lifetime::default())]
    lifetime: Lifetime,
    /// Command to run under elevated privileges
    command: Vec<String>,
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let config = get_gcloud_config();
    let access_token =
        get_access_token(&config, &args.service_account, &args.lifetime, &args.scopes);
    dbg!(args);
    dbg!(access_token);
    // dbg!(config);
    // service account
    // optional scopes
    // lifetime
}
