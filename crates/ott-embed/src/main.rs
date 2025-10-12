use std::time::Duration;

use ott_embed::pg_client::PgClient;
use ott_embed::tei_client::TextEmbedding;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::interval,
};

use tokio_stream::StreamExt;
use tracing::{debug, error, warn};
use tracing_subscriber::EnvFilter;

use fluvio::{consumer::ConsumerConfigExtBuilder, Fluvio, Offset};
use ott_types::{Embedding, Post};

const TEI_URL: &str = "http://tei-host-service:8080";
const TOPIC: &str = "posts";
const PARTITION: u32 = 0;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(true) // Colors enabled (default)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let (embed_tx, embed_rx) = tokio::sync::mpsc::channel::<Post>(1000);
    let (store_tx, store_rx) = tokio::sync::mpsc::channel::<Embedding>(1000);

    let read_task = tokio::spawn(async { read_task(embed_tx).await });
    let embed_task = tokio::spawn(async { embed_task(embed_rx, store_tx).await });
    let store_task = tokio::spawn(async { store_task(store_rx).await });

    let _result = tokio::join!(read_task, embed_task, store_task);
}

async fn read_task(sink: Sender<Post>) {
    let fluvio = Fluvio::connect()
        .await
        .expect("Failed to connect to Fluvio");

    let config = ConsumerConfigExtBuilder::default()
        .topic(TOPIC)
        .partition(PARTITION)
        .offset_start(Offset::beginning())
        .build()
        .expect("Failed to build consumer config");
    let mut stream = fluvio
        .consumer_with_config(config)
        .await
        .expect("Failed to create consumer");

    warn!("Ready to start consuming posts");
    while let Some(message) = stream.next().await
       && let Ok(record) = message
    {
        let post: Post = serde_json::from_slice(record.value()).expect("Invalid post message");
        sink.send(post).await.expect("Failed to internally send post");
    }
}

async fn embed_task(mut posts: Receiver<Post>, sink: Sender<Embedding>) {
    let tei_client = TextEmbedding::new(TEI_URL);

    warn!("Ready to start embedding posts");
    while let Some(post) = posts.recv().await {
        let embedding = tei_client.embed(&post.text).await;
        match embedding {
            Ok(vec) => {
                sink.send(Embedding {
                    uri: post.uri,
                    vector: vec,
                })
                .await
                .expect("Failed to send embedding between tasks");
            }
            Err(e) => {
                error!("Failed to embed post! {} {}", post.uri, e);
            }
        };
    }
}

async fn store_task(mut embeddings: Receiver<Embedding>) {
    warn!("Ready to start storing embeddings");
    let batch_size = 100;
    let mut flush_timer = interval(Duration::from_millis(500));
    let pg_client = PgClient::new().await.expect("Failed to connect to db");

    let mut batch = Vec::with_capacity(batch_size);
    loop {
        tokio::select! {
            Some(record) = embeddings.recv() => {
                batch.push(record);
                if batch.len() >= batch_size {
                    if let Err(e) = pg_client.insert_embeddings(&batch).await {
                        error!("Insert error: {}", e);
                    }
                    flush_timer.reset();
                    batch.clear();
                    debug!("Inserted normally");
                }
            }

            // Flush periodically even if batch isn't full
            _ = flush_timer.tick() => {
                if !batch.is_empty() {
                    if let Err(e) = pg_client.insert_embeddings(&batch).await {
                        error!("Insert error: {}", e);
                    }
                    batch.clear();
                    debug!("Inserted flushed");
                }
            }

            // Channel closed
            else => {
                // Final flush
                if !batch.is_empty() {
                    if let Err(e) = pg_client.insert_embeddings(&batch).await {
                        error!("Final insert error: {}", e);
                    }
                }
                break;
            }
        }
    }
}
