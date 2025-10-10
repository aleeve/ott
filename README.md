# OTT on that topic

OTT is an over the top bluesky custom feed (in the making)

The flow is as follows:

1. Two fluvio connector services consume wss streams, one for raw-posts and one for raw-likes
2. ott-filter consumes the keyed posts, and likes streams and keeps count on likes and other filters.
  It sends the passing posts to the fluvio topic posts.
3. ott-embed consumes the posts topic, embeds them  with tei running on host and stores the vectors in a pg cluster
4. ott-xrpc listens to getFeedSkeleton requests, gets the users last likeed post and gets similar posts from the pg db.

Still work in progress, especially the ott-xrpc service isn't fleshed out yet. Also I intend to add a VIP stream so that all posts 
liked by a feed user are guaranteed to pass the filter.

Then... The original intention was to use FASTopic to get topic vectors for every day in the semantic space and sample within the most relevant topics.
This is still the intention.

# Setup

## Install dependencies

```shell
# Install k8s tooling
brew install kind helm skaffold

# Install fvm and fluvio cli
curl -fsS https://hub.infinyon.cloud/install/install.sh | bash

# Install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install tei
cargo install --git https://github.com/huggingface/text-embeddings-inference

```

## Create a cluster

```shell
kind create cluster --config kind-cluster.yaml
```

# Run

```shell
skaffold run
```
