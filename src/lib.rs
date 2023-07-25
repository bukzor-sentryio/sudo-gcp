use std::{
    collections::HashSet,
    fmt::Display,
    process::{Command, Stdio},
    str::FromStr,
    time::Duration,
};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use keyring::Entry;
use reqwest::{blocking::Client, header::HeaderMap};
use serde::{Deserialize, Serialize};

const DEFAULT_OAUTH_SCOPES: &[&str] = &[
    "openid",
    "https://www.googleapis.com/auth/cloud-platform",
    "profile",
    "email",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
];

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

impl From<String> for AccessToken {
    fn from(value: String) -> Self {
        Self(value)
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
pub struct Delegates(Vec<String>);

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lifetime(u64);

// impl Serialize for Lifetime {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         serializer.serialize_u64(self.0.as_secs())
//     }
// }

// impl FromStr for Lifetime {
//     type Err = String;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let trimmed_s = s.trim_end_matches('s');
//         let seconds: u64 = trimmed_s.parse::<u64>().expect("failed to convert number");
//         Ok(Self(Duration::from_secs(seconds)))
//     }
// }

// impl From<u64> for Lifetime {
//     fn from(value: u64) -> Self {
//         Self(Duration::from_secs(value))
//     }
// }

impl Display for Lifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}s", self.0)
    }
}

impl Default for Lifetime {
    fn default() -> Self {
        Self(DEFAULT_LIFETIME_SECONDS)
    }
}

pub fn get_gcloud_config() -> anyhow::Result<GcloudConfig> {
    let proc = Command::new("gcloud")
        .args([
            "config",
            "config-helper",
            "--format",
            "csv[no-heading](configuration.properties.core.account,credential.access_token)",
        ])
        .stderr(Stdio::inherit())
        .output()
        .expect("gcloud call failed");

    let text = std::str::from_utf8(&proc.stdout)?;
    let config = GcloudConfig::from_str(text).map_err(|e| anyhow!(e));
    dbg!(&config);
    config
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct TokenRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    delegates: Option<Delegates>,
    lifetime: String,
    scope: Scopes,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenResponse {
    delegates: Option<Delegates>,
    access_token: AccessToken,
    expire_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StoredSecret {
    access_token: AccessToken,
    scopes: Scopes,
    expire_time: DateTime<Utc>,
}

pub fn get_access_token(
    gcloud_config: &GcloudConfig,
    delegates: Option<Delegates>,
    service_account: &Email,
    lifetime: &Lifetime,
    scopes: &Scopes,
) -> anyhow::Result<AccessToken> {
    let stored_secret = get_token_from_keyring(service_account);
    match stored_secret {
        Ok(s) => {
            if &s.scopes != scopes {
                println!("Scopes are not equal, getting a new token!");
                let new_token = get_token_from_gcloud(
                    service_account,
                    delegates,
                    lifetime,
                    scopes,
                    gcloud_config,
                )?;
                save_token_to_keyring(service_account, &new_token)?;
                return Ok(new_token.access_token);
            }

            if s.expire_time <= Utc::now() {
                println!("Token has expired, getting a new one!");
                let new_token = get_token_from_gcloud(
                    service_account,
                    delegates,
                    lifetime,
                    scopes,
                    gcloud_config,
                )?;
                save_token_to_keyring(service_account, &new_token)?;
                return Ok(new_token.access_token);
            }

            // TODO: check for delegates equality

            return Ok(s.access_token);
        }
        Err(e) => match e {
            keyring::Error::NoEntry => {
                let new_token = get_token_from_gcloud(
                    service_account,
                    delegates,
                    lifetime,
                    scopes,
                    gcloud_config,
                )?;
                save_token_to_keyring(service_account, &new_token)?;
                return Ok(new_token.access_token);
            }
            other_error => panic!("failed to get access token: {:?}", other_error),
        },
    }
}

fn get_token_from_gcloud(
    service_account: &Email,
    delegates: Option<Delegates>,
    lifetime: &Lifetime,
    scopes: &Scopes,
    gcloud_config: &GcloudConfig,
) -> anyhow::Result<StoredSecret> {
    let client: Client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(15))
        .build()?;

    let url = format!(
        "{}/projects/-/serviceAccounts/{}:generateAccessToken",
        IAM_API, service_account
    );

    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::ACCEPT, "application/json".parse()?);

    let token_request = TokenRequest {
        lifetime: format!("{}", lifetime),
        scope: scopes.clone(),
        delegates,
    };

    let request = client
        .post(url)
        .bearer_auth(gcloud_config.access_token.as_ref())
        .headers(headers)
        .json(&token_request);

    let response = request.send()?;
    let response_text = response.text()?;
    let token_response: TokenResponse = match serde_json::from_str(&response_text) {
        Ok(r) => r,
        Err(err) => {
            return Err(anyhow!(
                "{}: failed to parse response:\n\t{}",
                err,
                response_text.replace("\n", "\n\t"),
            ));
        }
    };

    Ok(StoredSecret {
        access_token: token_response.access_token,
        scopes: scopes.clone(),
        expire_time: token_response.expire_time,
    })
}

fn get_token_from_keyring(service_account: &Email) -> Result<StoredSecret, keyring::Error> {
    let entry = Entry::new(env!("CARGO_PKG_NAME"), &service_account.0)?;
    match entry.get_password() {
        Ok(s) => {
            let stored_secret: StoredSecret =
                serde_json::from_str(&s).expect("failed to parse json from keyring");
            Ok(stored_secret)
        }
        Err(e) => Err(e),
    }
}

// fn delete_token_from_keyring(service_account: &Email) -> anyhow::Result<AccessToken> {
//     todo!()
// }

fn save_token_to_keyring(
    service_account: &Email,
    stored_secret: &StoredSecret,
) -> anyhow::Result<()> {
    println!("Saving token to OS keyring!");
    let secret_entry = serde_json::to_string(stored_secret)?;
    let entry = Entry::new(env!("CARGO_PKG_NAME"), &service_account.0)?;
    match entry.set_password(&secret_entry) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

// TODO: support delegate chains? https://cloud.google.com/iam/docs/reference/credentials/rest/v1/projects.serviceAccounts/generateAccessToken
