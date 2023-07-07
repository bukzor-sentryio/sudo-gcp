***sudo-gcp is currently in alpha stages! Expect breaking changes.***

# Sudo GCP

This tool enables with running Google Cloud commands with temporary elevated privileges, using
short-lived OAuth access tokens.

`sudo-gcp` securely caches access tokens using the operating system's secret-store/keychain, and
will reuse matching non-expired tokens on subsequent calls. 

## Setup

1. Define a service account to be the holder of your elevated privileges
1. Grant elevated privileges to that service account
1. Define who should be eligible to temporarily gain those privileges
   - We use a google group with a "role-gcp-sudo-" prefixed group name
1. Assign those users the `roles/iam.workloadIdentityUser` role, bound to that
   service account

## Installation

```sh
cargo install sudo-gcp
```

## Configuration

If both environment and file configuration sources exist, environment variables take precedence
over the configuration file.


### Configuration by File
Configuration can be done with a `sudo-gcp.toml` file in the current
working directory. See the [example configuration file](doc/example-config.toml) for more details.

A configuration file in a different location can be provided when running `sudo-gcp` with the 
`--config-file` option.

```sh
# create a minimal configuration file if it does not already exist
echo > sudo-gcp.toml 'service_account = "my-terraformer@my-project.iam.gserviceaccount.com"'
```


### Configuration by Environment

Configuration is also supported via environment variables prefixed with `SUDOGCP_`.

```sh
export SUDOGCP_SERVICE_ACCOUNT=my-terraformer@my-project.iam.gserviceaccount.com
```

## Usage
After [configuration](#Configuration), wrap commands that need elevated privileges with the
`sudo-gcp` command, similar in usage to [`sudo`](https://man7.org/linux/man-pages/man8/sudo.8.html).

Examples:
```sh
sudo-gcp gcloud compute instances list

terraform plan  # error: no permission to read tfstate
sudo-gcp !!     # try again, but with necessary privileges
```

For complete usage details, run `sudo-gcp --help`.
