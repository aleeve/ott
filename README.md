# OTT on that topic

OTT is an over the top bluesky custom feed (in the making)

... mostly scaffolding so far ...

# Setup

## Install dependencies

```shell
brew install kind helm skaffold fluvio
```

## Rust

```shell
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
