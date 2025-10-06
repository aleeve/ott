# OTT on that topic

OTT is an over the top bluesky custom feed (in the making)

... mostly scaffolding so far ...

# Setup

## Install dependencies

```shell
# Install k8s tooling
brew install kind helm skaffold

# Install fvm and fluvio cli
curl -fsS https://hub.infinyon.cloud/install/install.sh | bash

# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Create a cluster

```shell
kind create cluster --config kind-cluster.yaml
```

# Run

```shell
skaffold run
```
