# Milestones

# m0 -- MVP

- cli options
- documentation
- tests

# mFuture -- fine polish

- set a check for a minimun expiration (don't reuse a token if it's about to expire)
- handle service account delegate chains
  - workaround: wrap sudo-gcp an additional time for each segment of the chain
- 1. $PWD config
  1. $PWD/.. config
  1. etc.
  1. $XDG_CONFIG_HOME (default: $HOME/.config
  1. $XDG_CONFIG_DIRS (default: /etx/xdg)
