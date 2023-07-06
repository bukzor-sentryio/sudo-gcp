# Sudo GCP

This tool helps with running Google Cloud commands with temporary elevated
privileges.

## Setup

1. define a service account to be the holder of your elevated privileges
1. grant elevated privileges to that service account
1. define who should be elegible to temporarily gain those privileges
   - we use a google group with a "role-gcp-sudo-" prefixed group name
1. assign those users the `roles/iam.workloadIdentityUser` role, bound to that
   service account

## Usage

Currently only a configuration file is supported. After creating the necessary
configuration file (see [example config](doc/example-config.toml) for full
listing), wrap commands that need elevated privileges with the `sudo-gcp`
command, similar in usage to
[`sudo`](https://man7.org/linux/man-pages/man8/sudo.8.html).

```sh
cargo install sudo-gcp

echo > sudo-gcp.toml 'service_account = "my-service-account@my-project.iam.gserviceaccount.com"'
sudo-gcp terraform plan
sudo-gcp gcloud compute instances list
```

## Configuration

Currently the command only looks for a `sudo-gcp.toml` file in the current
working directory. More flexibility commong soon.
