use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use tokio_stream::StreamExt;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use fluvio::{
    consumer::{ConsumerConfigExtBuilder, ConsumerStream},
    Fluvio, Offset,
};
use moka::{ops::compute::Op, sync::Cache};
use ott_filter::tei_client::TextEmbedding;
use ott_types::{Commit, Like, Post, RawPost};

const LIKES_TOPIC: &str = "raw-likes";
const POSTS_TOPIC: &str = "raw-posts";
const PARTITION_NUM: u32 = 0;
const TEI_URL: &str = "http://localhost:8080";

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

    let posts_fut = get_topic_stream(POSTS_TOPIC, PARTITION_NUM, &fluvio);
    let like_fut = get_topic_stream(LIKES_TOPIC, PARTITION_NUM, &fluvio);
    let (mut posts_stream, mut like_stream) = tokio::join!(posts_fut, like_fut);

    let (embed_tx, embed_rx) = mpsc::channel::<Post>(1000);

    // Start embedding tracing_subscriber
    let fut = async move {
        embed_post(embed_rx, store_tx).await;
    };
    tokio::spawn(fut);

    loop {
        let pcc = posts_cache.clone();
        let lcc = posts_cache.clone();
        select! {
            Some(Ok(record)) = posts_stream.next() => {
                let post: RawPost = serde_json::from_slice(record.value()).unwrap();
                match &post.commit {
                    Commit::Create{record} => {
                        pcc.entry(post.uri.clone())
                            .and_compute_with(|maybe_entry| {
                                    if maybe_entry.is_some() {
                                        Op::Nop
                                    } else {
                                        let post = Post{
                                            uri: post.uri,
                                            did: post.did,
                                            text: record.text.to_string(),
                                            ..Default::default()};
                                        Op::Put(post) // Insert
                                    }
                            }
                        );
                    },
                    _ => {
                        info!("Got create or update post");
                    }
                }

            },
            Some(Ok(record)) = like_stream.next() => {
                if let Ok(like) = serde_json::from_slice::<Like>(record.value()) {
                    lcc.entry(like.uri)
                        .and_compute_with(|maybe_entry| {
                            if let Some(entry) = maybe_entry {
                                let mut post = entry.into_value();
                                if post.count < 20 {
                                    post.count +=1;
                                    Op::Put(post)
                                } else {
                                    let tx_clone = embed_tx.clone();
                                    tokio::spawn(async move {
                                        if let Err(e) = tx_clone.send(post).await {
                                            error!("Failed to send post: {}", e);
                                        }
                                    });
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

async fn get_topic_stream(topic: &str, partition: u32, fluvio: &Fluvio) -> impl ConsumerStream {
    let config = ConsumerConfigExtBuilder::default()
        .topic(topic)
        .partition(partition)
        .offset_start(Offset::beginning())
        .build()
        .expect("Failed to build consumer config");
    let posts_stream = fluvio
        .consumer_with_config(config)
        .await
        .expect("Failed to create consumer");
    posts_stream
}

struct PostEmbedding {
    uri: String,
    vector: Vec<f32>,
}

async fn embed_post(mut post_rx: Receiver<Post>, embedding_tx: Sender<PostEmbedding>) {
    let tei_client = TextEmbedding::new(TEI_URL);
    while let Some(post) = post_rx.recv().await {
        if let Ok(vec) = tei_client.embed(post.text.as_str()).await {
            embedding_tx
                .send(PostEmbedding {
                    uri: post.uri,
                    vector: vec,
                })
                .await
                .expect("Failed to send vector");
        };
    }
}
