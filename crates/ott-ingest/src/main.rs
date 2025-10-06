use serde::Deserialize;
use tokio::{
    select,
    time::{Duration, sleep},
};
use tokio_stream::StreamExt;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{EnvFilter, field::display, fmt};

use fluvio::{
    Fluvio, Offset,
    consumer::{ConsumerConfigExtBuilder, ConsumerRecord},
};
use moka::{
    ops::compute::{CompResult, Op},
    sync::Cache,
};
use ott_ingest::tei_client;

const LIKES_TOPIC: &str = "raw-likes";
const POSTS_TOPIC: &str = "raw-posts";
const PARTITION_NUM: u32 = 0;

#[derive(Debug, Deserialize, Clone)]
struct Post {
    did: String,
    uri: String,
    #[serde(default)]
    count: u32,
}

#[derive(Debug, Deserialize, Clone)]
struct Like {
    did: String,
    uri: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(true) // Colors enabled (default)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let posts_cache: Cache<String, Post> = Cache::builder()
        .time_to_live(Duration::from_secs(60 * 60))
        .build();

    let fluvio = Fluvio::connect()
        .await
        .expect("Failed to connect to Fluvio");

    let config = ConsumerConfigExtBuilder::default()
        .topic(POSTS_TOPIC)
        .partition(PARTITION_NUM)
        .offset_start(Offset::beginning())
        .build()
        .expect("Failed to build consumer config");
    let mut posts_stream = fluvio
        .consumer_with_config(config)
        .await
        .expect("Failed to create consumer");

    let config = ConsumerConfigExtBuilder::default()
        .topic(LIKES_TOPIC)
        .partition(PARTITION_NUM)
        .offset_start(Offset::beginning())
        .build()
        .expect("Failed to build consumer config");
    let mut like_stream = fluvio
        .consumer_with_config(config)
        .await
        .expect("Failed to create consumer");

    loop {
        let pcc = posts_cache.clone();
        let lcc = posts_cache.clone();
        select! {
            Some(Ok(record)) = posts_stream.next() => {
                let post: Post = serde_json::from_slice(record.value()).unwrap();
                pcc.entry(post.uri.clone())
                    .and_compute_with(|maybe_entry| {
                        if maybe_entry.is_some() {
                            Op::Nop
                        } else {
                            Op::Put(post) // Insert
                        }
                });
            },
            Some(Ok(record)) = like_stream.next() => {
                if let Ok(like) = serde_json::from_slice::<Like>(record.value()) {
                    lcc.entry(like.uri)
                        .and_compute_with(|maybe_entry| {
                            if let Some(entry) = maybe_entry {
                                let mut post = entry.into_value();
                                if post.count < 20 {
                                    post.count +=1;
                                    warn!("Incread counter for {:#?}", post);
                                    Op::Put(post)
                                } else {
                                    Op::Remove
                                }
                            } else {
                                Op::Nop // Skip as post is out of cache
                            }
                    });
                 } else {
                     warn!("Failed deserializing, likely not like commit");
                };
            }
        }
    }
}
