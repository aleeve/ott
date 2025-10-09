use anyhow::Result;
use atproto_identity::{
    config::{default_env, optional_env, require_env, version, DnsNameservers},
    key::{generate_key, identify_key, to_public, KeyType},
};
use atproto_xrpcs::authorization::ResolvingAuthorization;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use clap::Parser;
use http::{HeaderMap, StatusCode};
use ott_xrpc::webcontext::{ContextConfig, ServiceDID, ServiceDocument, WebContext};
use serde::Deserialize;
use serde_json::json;

/// AT Protocol XRPC Hello World Service
#[derive(Parser)]
#[command(
    name = "atproto-xrpcs-helloworld",
    version,
    about = "AT Protocol XRPC Hello World demonstration service",
    long_about = "
A demonstration XRPC service implementation showcasing the AT Protocol ecosystem.
This service provides a simple \"Hello, World!\" endpoint that supports both
authenticated and unauthenticated requests.

FEATURES:
  - AT Protocol identity resolution and DID document management
  - XRPC service endpoint with optional authentication
  - DID:web identity publishing via .well-known endpoints
  - JWT-based request authentication using AT Protocol standards

ENVIRONMENT VARIABLES:
  SERVICE_KEY          Private key for service identity (required)
  EXTERNAL_BASE        External hostname for service endpoints (required)
  PORT                HTTP server port (default: 8080)
  PLC_HOSTNAME        PLC directory hostname (default: plc.directory)
  USER_AGENT          HTTP User-Agent header (auto-generated)
  DNS_NAMESERVERS     Custom DNS nameservers (optional)
  CERTIFICATE_BUNDLES Additional CA certificates (optional)

ENDPOINTS:
  GET /                           HTML index page
  GET /.well-known/did.json       DID document (DID:web)
  GET /.well-known/atproto-did    AT Protocol DID identifier
  GET /xrpc/.../Hello            Hello World XRPC endpoint
"
)]
struct Args {}

#[tokio::main]
async fn main() -> Result<()> {
    let _args = Args::parse();

    let plc_hostname = default_env("PLC_HOSTNAME", "plc.directory");

    let external_base = require_env("EXTERNAL_BASE")?;
    let port = default_env("PORT", "8080");
    let service_did = format!("did:web:{}", external_base);
    let dns_nameservers: DnsNameservers = optional_env("DNS_NAMESERVERS").try_into()?;

    let private_service_key = generate_key(KeyType::P256Private)?.to_string();

    let private_service_key_data = identify_key(&private_service_key)?;
    let public_service_key_data = to_public(&private_service_key_data)?;
    let public_service_key = public_service_key_data.to_string();
    let default_user_agent = format!(
        "atproto-identity-rs ({}; +https://tangled.sh/@smokesignal.events/atproto-identity-rs)",
        version()?
    );
    let user_agent = default_env("USER_AGENT", &default_user_agent);

    let config = ContextConfig {
        public_service_key,
        private_service_key_data,
        service_did,
        plc_hostname,
        external_base,
        dns_nameservers,
        user_agent,
    };
    let web_context = WebContext::new(config).unwrap();

    let router = Router::new()
        .route("/", get(handle_index))
        .route("/.well-known/did.json", get(handle_wellknown_did_web))
        .route(
            "/.well-known/atproto-did",
            get(handle_wellknown_atproto_did),
        )
        .route(
            "/xrpc/garden.lexicon.ngerakines.helloworld.Hello",
            get(handle_xrpc_hello_world),
        )
        .with_state(web_context);

    let bind_address = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;

    // Start the web server in the background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, router).await {
            eprintln!("Server error: {}", e);
        }
    });

    println!(
        "XRPC Hello World service started on http://0.0.0.0:{}",
        port
    );

    // Keep the server running
    server_handle.await.unwrap();

    Ok(())
}

async fn handle_index() -> Html<&'static str> {
    Html("<html><body><h1>Right place wrong protocol...</h1></body></html>")
}

// /.well-known/did.json
async fn handle_wellknown_did_web(
    service_document: State<ServiceDocument>,
) -> Json<serde_json::Value> {
    Json(service_document.0 .0)
}

// /.well-known/atproto-did
async fn handle_wellknown_atproto_did(service_did: State<ServiceDID>) -> Response {
    (StatusCode::OK, service_did.0 .0).into_response()
}

#[derive(Deserialize)]
struct HelloParameters {
    subject: Option<String>,
}

// /xrpc/garden.lexicon.ngerakines.helloworld.Hello
async fn handle_xrpc_hello_world(
    parameters: Query<HelloParameters>,
    headers: HeaderMap,
    authorization: Option<ResolvingAuthorization>,
) -> Json<serde_json::Value> {
    println!("headers {headers:?}");
    let subject = parameters.subject.as_deref().unwrap_or("World");
    let message = if let Some(auth) = authorization {
        format!("Hello, authenticated {}! (caller: {})", subject, auth.3)
    } else {
        format!("Hello, {}!", subject)
    };
    Json(json!({ "message": message }))
}
