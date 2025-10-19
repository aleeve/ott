use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use jacquard::api::app_bsky::actor::get_profiles::GetProfiles;
use jacquard::client::Agent;
use jacquard::client::AgentSessionExt;
use jacquard::client::BasicClient;
use jacquard::client::MemorySessionStore;
use jacquard::from_data_owned;
use jacquard::types::ident::AtIdentifier;
use jacquard::types::nsid::Nsid;
use jacquard::xrpc::XrpcExt;
use jacquard::CowStr;
use jacquard_api::app_bsky::feed::post::Post;
use jacquard_api::com_atproto::repo::list_records::ListRecords;
use tracing::{info, warn};
use url::Url;

// Super silly, there must be some good traits to use
type MyAgent = Agent<
    jacquard::client::credential_session::CredentialSession<
        MemorySessionStore<
            (jacquard::types::did::Did<'static>, CowStr<'static>),
            jacquard::client::AtpSession,
        >,
        Agent<
            jacquard::client::credential_session::CredentialSession<
                MemorySessionStore<
                    (jacquard::types::did::Did<'static>, CowStr<'static>),
                    jacquard::client::AtpSession,
                >,
                jacquard_identity::JacquardResolver,
            >,
        >,
    >,
>;
pub struct BskyClient {
    pub agent: MyAgent,
    pub base_url: Url,
}

impl BskyClient {
    pub async fn new() -> Result<Self> {
        let app_did = std::env::var("APP_DID").expect("Need to set APP_DID");
        let app_key = std::env::var("APP_KEY").expect("Need to set APP_KEY");
        let base = url::Url::parse("https://public.api.bsky.app")?;
        let session = jacquard::client::credential_session::CredentialSession::new(
            Arc::new(MemorySessionStore::default()),
            Arc::new(BasicClient::default()),
        );
        session
            .login(
                CowStr::from(app_did),
                CowStr::from(app_key),
                None,
                None,
                None,
            )
            .await?;
        let token = session.access_token().await.unwrap();
        warn!("{:#?}", token);
        let agent = Agent::from(session);
        Ok(Self {
            agent,
            base_url: base,
        })
    }

    pub async fn get_like(&self, did: &str) -> Result<Post> {
        let request = ListRecords::new()
            .collection(Nsid::from_str("bsky.feed.like")?)
            .limit(1)
            .repo(AtIdentifier::from_str(did).expect("did to be ok"))
            .build();

        let response = self
            .agent
            .xrpc(self.base_url.clone())
            .send(&request)
            .await?;
        let data = response
            .into_output()
            .unwrap()
            .records
            .first()
            .unwrap()
            .to_owned();
        let post: Post = from_data_owned(data)?;

        Ok(post)
    }

    async fn get_profile(&self, did: &str) -> Result<()> {
        let request = GetProfiles::new()
            .actors(vec![AtIdentifier::Did(did.parse()?)])
            .build();
        let response = self
            .agent
            .xrpc(self.base_url.clone())
            .send(&request)
            .await?;
        info!("{:#?}", response.parse());
        Ok(())
    }

    async fn get_post(&self, uri: &str) -> Result<()> {
        let response = self.agent.get_record::<Post>(uri.parse()?).await?;
        let output = response.into_output()?;
        info!("{:#?}", output);

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    fn setup_tracing() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init()
            .ok();
    }

    #[fixture]
    async fn client() -> BskyClient {
        BskyClient::new().await.unwrap()
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_latest_like(
        #[values("did:plc:klugggc44dmpomjkuzyahzjd")] did: &str,
        #[future] client: BskyClient,
        _setup_tracing: (),
    ) {
        info!("Starting");
        let client = client.await;
        let post = client.get_like(did).await;
        assert!(post.is_ok());
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_profile(
        #[values("did:plc:klugggc44dmpomjkuzyahzjd", "did:plc:6u4att3krympska2rcfphobc")] did: &str,
        #[future] client: BskyClient,
        _setup_tracing: (),
    ) {
        info!("Starting");
        let did = client.await.get_profile(did).await;
        info!("{:#?}", did);

        assert!(did.is_ok());
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_post(#[future] client: BskyClient) {}
}
