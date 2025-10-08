use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct RawPost {
    pub did: String,
    pub uri: String,
    pub commit: Commit,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "operation", rename_all = "lowercase")]
pub enum Commit {
    Create { record: Record },
    Delete,
    Update,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Record {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Post {
    pub did: String,
    pub uri: String,
    pub text: String,
    pub count: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Like {
    pub did: String,
    pub uri: String,
}

#[derive(Debug, Clone)]
pub struct Embedding {
    pub uri: String,
    pub vector: Vec<f32>,
}
