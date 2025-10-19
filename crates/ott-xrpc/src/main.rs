use std::collections::BTreeMap;

use axum::{Json, Router};
use jacquard::types::did_doc::{DidDocument, Service, VerificationMethod};
use jacquard_api::app_bsky::feed::{
    get_feed_skeleton::{GetFeedSkeletonOutput, GetFeedSkeletonRequest},
    SkeletonFeedPost,
};
use jacquard_axum::did_web::did_web_router;
use jacquard_axum::ExtractXrpc;
use jacquard_axum::{
    service_auth::{ExtractServiceAuth, ServiceAuthConfig},
    IntoRouter,
};
use jacquard_common::types::string::Did;
use jacquard_identity::resolver::ResolverOptions;
use jacquard_identity::JacquardResolver;
use ott_xrpc::{bsky::BskyClient, key::generate_key};

use tracing::info;
use tracing_subscriber::EnvFilter;

async fn handler(
    ExtractServiceAuth(auth): ExtractServiceAuth,
    ExtractXrpc(args): ExtractXrpc<GetFeedSkeletonRequest>,
) -> Result<Json<GetFeedSkeletonOutput<'static>>, String> {
    let posts: Vec<SkeletonFeedPost<'static>> = vec![SkeletonFeedPost {
        post: "at://did:plc:klugggc44dmpomjkuzyahzjd/app.bsky.feed.post/3m2y6a5h6os27"
            .parse()
            .map_err(|_| "Failed to parse uri".to_string())?,
        feed_context: None,
        extra_data: BTreeMap::default(),
        reason: None,
    }];

    let output = GetFeedSkeletonOutput::<'static> {
        feed: posts,
        cursor: None,
        req_id: None,
        extra_data: BTreeMap::default(),
    };
    Ok(Json(output.clone()))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_ansi(true) // Colors enabled (default)
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Setup");
    let did_str = "did:web:ott.aleeve.dev";
    let did = Did::new_static(did_str);

    let verification_method = VerificationMethod {
        id: format!("{}#atproto", did_str).into(),
        r#type: "Mutlikey".into(),
        controller: Some("did:web:ott.aleeve.dev".into()),
        public_key_multibase: Some(generate_key().into()),
        extra_data: BTreeMap::default(),
    };

    let service = Service {
        id: "#bsky_fg".into(),
        service_endpoint: Some("https://ott.aleeve.dev/".into()),
        r#type: "BskyFeedGenerator".into(),
        extra_data: BTreeMap::default(),
    };

    let did_doc: DidDocument = DidDocument {
        id: did.clone().unwrap(),
        also_known_as: Some(vec!["at://ott.aleeve.dev".into()]),
        verification_method: Some(vec![verification_method]),
        service: Some(vec![service]),
        extra_data: BTreeMap::default(),
    };

    let resolver = JacquardResolver::new(reqwest::Client::new(), ResolverOptions::default());
    let config: ServiceAuthConfig<JacquardResolver> =
        ServiceAuthConfig::new(did.clone().unwrap(), resolver);

    let app = Router::new()
        .merge(GetFeedSkeletonRequest::into_router(handler))
        .with_state(config)
        .merge(did_web_router(did_doc));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    info!("Starting service");
    axum::serve(listener, app).await.unwrap();
}
