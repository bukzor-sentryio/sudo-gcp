use clap::Parser;

use sudo_gcp::{Email, Lifetime, Scopes};

#[derive(Debug, Parser)]
#[command(author, version)]
struct Args {
    /// Email of service account to impersonate
    service_account: Email,
    /// Comma separated list of oauth scopes
    #[arg(short, long, default_value_t = Scopes::default())]
    scopes: Scopes,
    /// Lifetime of access token in seconds
    #[arg(long, short, default_value_t = Lifetime::default())]
    lifetime: Lifetime,
}

fn main() {
    let args = Args::parse();
    dbg!(args);
    // service account
    // optional scopes
    // lifetime
}
