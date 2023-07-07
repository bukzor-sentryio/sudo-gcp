***sudo-gcp is currently in alpha stages! Expect breaking changes.***

# Sudo GCP

This tool helps with running Google Cloud commands with temporary elevated
privileges.

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

## Usage

For more usage details, run `sudo-gcp --help`.

After creating the necessary [configurations](#Configuration), wrap commands 
that need elevated privileges with the `sudo-gcp` command, similar in 
usage to [`sudo`](https://man7.org/linux/man-pages/man8/sudo.8.html).

```sh
cargo install sudo-gcp

echo > sudo-gcp.toml 'service_account = "my-service-account@my-project.iam.gserviceaccount.com"'
sudo-gcp terraform plan
sudo-gcp gcloud compute instances list
```

## Configuration


Configuration can be done with a `sudo-gcp.toml` file in the current
working directory. See the [example configuration file](doc/example-config.toml) for more details.

A configuration file in a different location can be provided when running `sudo-gcp` with the `--config-file` option.

Configuration is also supported via environment variables prefixed with `SUDOGCP_`.

If both configuration sources exist, environment variables take precedence over the configuration file.
