# Milestones

# m0 -- MVP

- cli options
- handle configuration of defaults via:
  1. env vars
- documentation
- tests
- Usage:
  ```
  export SUDOGCP_SERVICEACCOUNT=my-service-account@my-project.iam.gserviceaccount.com
  sudo-gcp terraform plan
  sudo-gcp gcloud compute instances list
  ```

# mFuture -- fine polish

- handle service account delegate chains
  - workaround: wrap sudo-gcp an additional time for each segment of the chain
- 1. $PWD config
  1. $PWD/.. config
  1. etc.
  1. $XDG_CONFIG_HOME (default: $HOME/.config
  1. $XDG_CONFIG_DIRS (default: /etx/xdg)
