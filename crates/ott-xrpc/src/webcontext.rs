use anyhow::Result;
use async_trait::async_trait;
use atproto_identity::config::{optional_env, CertificateBundles, DnsNameservers};
use atproto_identity::resolve::{HickoryDnsResolver, SharedIdentityResolver};
use atproto_identity::{
    key::{KeyData, KeyProvider},
    resolve::InnerIdentityResolver,
    storage_lru::LruDidDocumentStorage,
};
use serde_json::json;
use std::ops::Deref;
use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};

use atproto_identity::{resolve::IdentityResolver, storage::DidDocumentStorage};
use axum::extract::FromRef;

#[derive(Clone)]
pub struct SimpleKeyProvider {
    keys: HashMap<String, KeyData>,
}

impl Default for SimpleKeyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SimpleKeyProvider {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }
}

#[async_trait]
impl KeyProvider for SimpleKeyProvider {
    async fn get_private_key_by_id(&self, key_id: &str) -> anyhow::Result<Option<KeyData>> {
        Ok(self.keys.get(key_id).cloned())
    }
}

#[derive(Clone)]
pub struct ServiceDocument(pub serde_json::Value);

#[derive(Clone)]
pub struct ServiceDID(pub String);

pub struct InnerWebContext {
    pub http_client: reqwest::Client,
    pub document_storage: Arc<dyn DidDocumentStorage>,
    pub key_provider: Arc<dyn KeyProvider>,
    pub service_document: ServiceDocument,
    pub service_did: ServiceDID,
    pub identity_resolver: Arc<dyn IdentityResolver>,
}

#[derive(Clone, FromRef)]
pub struct WebContext(pub Arc<InnerWebContext>);

pub struct ContextConfig {
    pub public_service_key: String,
    pub private_service_key_data: KeyData,
    pub service_did: String,
    pub plc_hostname: String,
    pub external_base: String,
    pub dns_nameservers: DnsNameservers,
    pub user_agent: String,
}

impl WebContext {
    pub fn new(config: ContextConfig) -> Result<Self> {
        let signing_key_storage = HashMap::from_iter(vec![(
            config.public_service_key.clone(),
            config.private_service_key_data.clone(),
        )]);

        let certificate_bundles: CertificateBundles =
            optional_env("CERTIFICATE_BUNDLES").try_into()?;

        let mut client_builder = reqwest::Client::builder();
        for ca_certificate in certificate_bundles.as_ref() {
            let cert = std::fs::read(ca_certificate)?;
            let cert = reqwest::Certificate::from_pem(&cert)?;
            client_builder = client_builder.add_root_certificate(cert);
        }

        client_builder = client_builder.user_agent(config.user_agent);
        let http_client = client_builder.build()?;

        let dns_resolver = HickoryDnsResolver::create_resolver(config.dns_nameservers.as_ref());

        let service_did = config.service_did.clone();
        let external_base = config.external_base;
        let service_document = ServiceDocument(json!({
                "@context": vec!["https://www.w3.org/ns/did/v1","https://w3id.org/security/multikey/v1"],
                "id": service_did,
                "verificationMethod":[{
                    "id": format!("{service_did}#atproto"),
                    "type":"Multikey",
                    "controller": service_did,
                    "publicKeyMultibase": config.public_service_key
                }],
                "service":[{
                    "id":"#bsky_fg",
                    "type":"BskyFeedGenerator",
                    "serviceEndpoint":format!("https://{external_base}")
                }]
            }
        ));

        let service_did = ServiceDID(config.service_did);

        let identity_resolver = Arc::new(SharedIdentityResolver(Arc::new(InnerIdentityResolver {
            dns_resolver: Arc::new(dns_resolver),
            http_client: http_client.clone(),
            plc_hostname: config.plc_hostname,
        })));

        let web_context = Self(Arc::new(InnerWebContext {
            http_client: http_client.clone(),
            document_storage: Arc::new(LruDidDocumentStorage::new(NonZeroUsize::new(255).unwrap())),
            key_provider: Arc::new(SimpleKeyProvider {
                keys: signing_key_storage,
            }),
            service_document,
            service_did,
            identity_resolver,
        }));
        Ok(web_context)
    }
}

impl Deref for WebContext {
    type Target = InnerWebContext;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRef<WebContext> for reqwest::Client {
    fn from_ref(context: &WebContext) -> Self {
        context.0.http_client.clone()
    }
}

impl FromRef<WebContext> for ServiceDocument {
    fn from_ref(context: &WebContext) -> Self {
        context.0.service_document.clone()
    }
}

impl FromRef<WebContext> for ServiceDID {
    fn from_ref(context: &WebContext) -> Self {
        context.0.service_did.clone()
    }
}

impl FromRef<WebContext> for Arc<dyn DidDocumentStorage> {
    fn from_ref(context: &WebContext) -> Self {
        context.0.document_storage.clone()
    }
}

impl FromRef<WebContext> for Arc<dyn KeyProvider> {
    fn from_ref(context: &WebContext) -> Self {
        context.0.key_provider.clone()
    }
}

impl FromRef<WebContext> for Arc<dyn IdentityResolver> {
    fn from_ref(context: &WebContext) -> Self {
        context.0.identity_resolver.clone()
    }
}
