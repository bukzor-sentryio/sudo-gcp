# Milestones

# m0 -- MVP

- cli options
- documentation
- tests

# mFuture -- fine polish

- set a check for a minimun expiration (don't reuse a token if it's about to
  expire)
- handle service account delegate chains
  - workaround: wrap sudo-gcp an additional time for each segment of the chain
- 1. $PWD config
  1. $PWD/.. config
  1. etc.
  1. $XDG_CONFIG_HOME (default: $HOME/.config
  1. $XDG_CONFIG_DIRS (default: /etx/xdg)

# tests

- end-to-end (network access)
  - hit "tokeninfo" endpoint with $CLOUDSDK_AUTH_ACCESS_TOKEN
  - hit "tokeninfo" endpoint with $GOOGLE_OAUTH_ACCESS_TOKEN
- networked integration
  - get-gcloud-config
  - get-access-token
- cli-scope (network mocked?)
  - --version behavior
  - --help behavior
  - config via file
  - config via env
  - ? config via `-u`
- function-scope
  - get-settings
  - ? config via `-u`
- struct scope
  - Lifetime
  - StoredSecret
  - AccessToken
  - GcloudConfig
  - Email
  - Scopes
