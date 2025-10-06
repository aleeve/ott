use std::{fs, path::Path};

use atproto_identity::resolve::HickoryDnsResolver;
use atproto_lexicon::resolve::{DefaultLexiconResolver, LexiconResolver};
use atproto_lexicon::resolve_recursive::{RecursiveLexiconResolver, RecursiveResolverConfig};

fn main() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let http_client = reqwest::Client::new();
        let dns_resolver = HickoryDnsResolver::create_resolver(&[]);
        let resolver = DefaultLexiconResolver::new(http_client, dns_resolver);

        let config = RecursiveResolverConfig {
            max_depth: 5,        // Maximum recursion depth
            include_entry: true, // Include the entry lexicon in results
        };

        let recursive_resolver = RecursiveLexiconResolver::with_config(resolver, config);

        let lexicon = recursive_resolver
            .resolve_recursive("app.bsky.feed.post")
            .await?;
        let json = serde_json::to_string_pretty(&lexicon)?;
        let out_dir = std::env::var("OUT_DIR")?;
        let dest_path = Path::new(&out_dir).join("post.json");
        fs::write(&dest_path, json)?;

        let lexicon = recursive_resolver
            .resolve_recursive("app.bsky.feed.like")
            .await?;
        let json = serde_json::to_string_pretty(&lexicon)?;
        let out_dir = std::env::var("OUT_DIR")?;
        let dest_path = Path::new(&out_dir).join("like.json");
        fs::write(&dest_path, json)?;

        Ok(())
    })
}
