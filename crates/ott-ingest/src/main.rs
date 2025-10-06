use tokio::{
    select,
    time::{Duration, sleep},
};
use tokio_stream::StreamExt;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};

use fluvio::{
    Fluvio, Offset,
    consumer::{ConsumerConfigExtBuilder, ConsumerRecord},
};
use moka::{
    ops::compute::{CompResult, Op},
    sync::Cache,
};
use ott_ingest::at_types;
use ott_ingest::tei_client;

const LIKES_TOPIC: &str = "raw-likes";
const POSTS_TOPIC: &str = "raw-posts";
const PARTITION_NUM: u32 = 0;

#[derive(Clone)]
struct BskyPost {
    likes: u32,
    uri: String,
    did: String,
    text: String,
}

impl BskyPost {}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(true) // Colors enabled (default)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let posts_cache: Cache<String, u32> = Cache::builder()
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
            result = like_stream.next() => {
                match result {
                    Some(Ok(record)) => {
                        record.value();
                        pcc.entry(String::from("hej"))
                            .and_compute_with(|maybe_entry| {
                                if let Some(entry) = maybe_entry {
                                    let counter = entry.into_value();
                                    if counter < 2 {
                                        Op::Put(counter.saturating_add(1)) // Update
                                    } else {
                                        Op::Remove
                                    }
                                } else {
                                    Op::Put(1) // Insert
                                }
                        });
                    }
                    Some(Err(e)) => {todo!()}
                    None => {todo!()}
                }
            }

            Some(Ok(result)) = posts_stream.next() => {
                lcc.entry(String::from("hej"))
                    .and_compute_with(|maybe_entry| {
                        if let Some(entry) = maybe_entry {
                            let counter = entry.into_value();
                            if counter < 2 {
                                Op::Put(counter.saturating_add(1)) // Update
                            } else {
                                Op::Remove
                            }
                        } else {
                            Op::Nop // Skip as post is out of cache
                        }
                });
            }
        }
    }
}
