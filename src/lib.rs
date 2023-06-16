#![allow(dead_code)]
#![allow(unused_variables)]

use std::{error::Error, fmt::Display, process::Command, str::FromStr, time::Duration};

const DEFAULT_OAUTH_SCOPES: &[&str] = &[
    "openid",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/appengine.admin",
    "https://www.googleapis.com/auth/sqlservice.login",
    "https://www.googleapis.com/auth/compute",
    "https://www.googleapis.com/auth/gmail.settings.basic",
    "https://www.googleapis.com/auth/gmail.settings.sharing",
    "https://www.googleapis.com/auth/chrome.management.policy",
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/admin.directory.customer",
    "https://www.googleapis.com/auth/admin.directory.domain",
    "https://www.googleapis.com/auth/admin.directory.group",
    "https://www.googleapis.com/auth/admin.directory.orgunit",
    "https://www.googleapis.com/auth/admin.directory.rolemanagement",
    "https://www.googleapis.com/auth/admin.directory.userschema",
    "https://www.googleapis.com/auth/admin.directory.user",
    "https://www.googleapis.com/auth/apps.groups.settings",
];
const DEFAULT_LIFETIME_SECONDS: u64 = 3600;
const IAM_API: &str = "https://iamcredentials.googleapis.com/v1";

#[derive(Debug)]
pub struct GcloudConfig {
    account: String,
    access_token: String,
}

#[derive(Debug, Clone)]
pub struct Email(String);

impl FromStr for Email {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct Scopes(Vec<String>);

impl FromStr for Scopes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let scopes = s.split(',').map(|s| s.to_string()).collect();
        Ok(Self(scopes))
    }
}

impl Display for Scopes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scopes = self.0.join(",");
        write!(f, "{}", scopes)
    }
}

#[derive(Debug, Clone)]
pub struct Lifetime(Duration);

impl FromStr for Lifetime {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let seconds: u64 = u64::from_str_radix(s, 10).expect("failed to convert number");
        Ok(Self(Duration::from_secs(seconds)))
    }
}

impl Display for Lifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_secs())
    }
}

impl Default for Lifetime {
    fn default() -> Self {
        Self(Duration::from_secs(DEFAULT_LIFETIME_SECONDS))
    }
}

#[warn(unused_variables)]

impl Default for Scopes {
    fn default() -> Self {
        let owned_scopes: Vec<String> = DEFAULT_OAUTH_SCOPES
            .iter()
            .map(|scope| scope.to_string())
            .collect();
        Self(owned_scopes)
    }
}

pub fn get_gcloud_config() -> GcloudConfig {
    Command::new("gcloud")
        .args([
            "config",
            "config-helper",
            "--format",
            "json(configuration.properties.core.account,credential.access_token)",
        ])
        .output()
        .expect("gcloud call failed");
    todo!()
}

pub fn get_access_token(gcloud_config: GcloudConfig, service_account: Email) {}
