use std::{
    collections::HashSet,
    fmt::Display,
    process::{Command, Stdio},
    str::FromStr,
    time::Duration,
};

use chrono::{DateTime, Utc};
use keyring::Entry;
use reqwest::{blocking::Client, header::HeaderMap};
use serde::{Deserialize, Serialize};

// const DEFAULT_OAUTH_SCOPES: &[&str] = &[
//     "openid",
//     "https://www.googleapis.com/auth/userinfo.email",
//     "https://www.googleapis.com/auth/userinfo.profile",
//     "https://www.googleapis.com/auth/cloud-platform",
//     "https://www.googleapis.com/auth/appengine.admin",
//     "https://www.googleapis.com/auth/sqlservice.login",
//     "https://www.googleapis.com/auth/compute",
//     "https://www.googleapis.com/auth/gmail.settings.basic",
//     "https://www.googleapis.com/auth/gmail.settings.sharing",
//     "https://www.googleapis.com/auth/chrome.management.policy",
//     "https://www.googleapis.com/auth/cloud-platform",
//     "https://www.googleapis.com/auth/admin.directory.customer",
//     "https://www.googleapis.com/auth/admin.directory.domain",
//     "https://www.googleapis.com/auth/admin.directory.group",
//     "https://www.googleapis.com/auth/admin.directory.orgunit",
//     "https://www.googleapis.com/auth/admin.directory.rolemanagement",
//     "https://www.googleapis.com/auth/admin.directory.userschema",
//     "https://www.googleapis.com/auth/admin.directory.user",
//     "https://www.googleapis.com/auth/apps.groups.settings",
// ];

const DEFAULT_OAUTH_SCOPES: &[&str] = &["https://www.googleapis.com/auth/cloud-platform"];

const DEFAULT_LIFETIME_SECONDS: u64 = 3600;
const IAM_API: &str = "https://iamcredentials.googleapis.com/v1";
static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessToken(String);

impl FromStr for AccessToken {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub struct GcloudConfig {
    _account: String,
    access_token: AccessToken,
}

impl FromStr for GcloudConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (account, access_token) = s.trim().split_once(',').expect("config-helper call failed");
        Ok(Self {
            _account: account.to_string(),
            access_token: AccessToken::from_str(access_token)
                .expect("failed to parse access token"),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email(String);

impl FromStr for Email {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scopes(HashSet<String>);

impl FromStr for Scopes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let scopes = s.split(',').map(|s| s.to_string()).collect();
        Ok(Self(scopes))
    }
}

impl Display for Scopes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sorted_scopes: Vec<String> = self.0.iter().map(|s| s.to_string()).collect();
        let scopes: String = sorted_scopes.join(",");
        write!(f, "{}", scopes)
    }
}
impl Default for Scopes {
    fn default() -> Self {
        let owned_scopes: HashSet<String> = DEFAULT_OAUTH_SCOPES
            .iter()
            .map(|scope| scope.to_string())
            .collect();
        Self(owned_scopes)
    }
}

impl Scopes {
    pub fn append_scopes(&self, additional_scopes: Scopes) -> Self {
        let mut scopes = Scopes::default();
        scopes.0.extend(additional_scopes.0);
        scopes
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Lifetime(Duration);

impl Serialize for Lifetime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for Lifetime {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed_s = s.trim_end_matches('s');
        let seconds: u64 = trimmed_s.parse::<u64>().expect("failed to convert number");
        Ok(Self(Duration::from_secs(seconds)))
    }
}

impl Display for Lifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}s", self.0.as_secs())
    }
}

impl Default for Lifetime {
    fn default() -> Self {
        Self(Duration::from_secs(DEFAULT_LIFETIME_SECONDS))
    }
}

pub fn get_gcloud_config() -> GcloudConfig {
    let config = Command::new("gcloud")
        .args([
            "config",
            "config-helper",
            "--format",
            "csv[no-heading](configuration.properties.core.account,credential.access_token)",
        ])
        .stderr(Stdio::inherit())
        .output()
        .expect("gcloud call failed");
    GcloudConfig::from_str(std::str::from_utf8(&config.stdout).unwrap()).unwrap()
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct TokenRequest {
    lifetime: Lifetime,
    scope: Scopes,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenResponse {
    access_token: AccessToken,
    expire_time: DateTime<Utc>,
}

pub fn get_access_token(
    gcloud_config: &GcloudConfig,
    service_account: &Email,
    lifetime: &Lifetime,
    scopes: &Scopes,
) -> AccessToken {
    let client: Client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();

    let url = format!(
        "{}/projects/-/serviceAccounts/{}:generateAccessToken",
        IAM_API, service_account
    );

    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::ACCEPT, "application/json".parse().unwrap());

    let token_request = TokenRequest {
        lifetime: lifetime.clone(),
        scope: scopes.clone(),
    };

    let request = client
        .post(url)
        .bearer_auth(gcloud_config.access_token.as_ref())
        .headers(headers)
        .json(&token_request);

    let response: TokenResponse = request.send().unwrap().json().unwrap();
    save_token_to_keyring(service_account, &response).unwrap();
    response.access_token
}

fn save_token_to_keyring(
    service_account: &Email,
    token_response: &TokenResponse,
) -> keyring::Result<()> {
    let secret_entry =
        serde_json::to_string(token_response).expect("failed to serialize json response to string");
    let entry = Entry::new(env!("CARGO_PKG_NAME"), &service_account.0)?;
    entry.set_password(&secret_entry)
}

// TODO: support delegate chains? https://cloud.google.com/iam/docs/reference/credentials/rest/v1/projects.serviceAccounts/generateAccessToken
