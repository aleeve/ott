use anyhow::Result;
use ott_types::Embedding;
use pgvector::Vector;
use sqlx::PgPool;

pub struct PgClient {
    pool: PgPool,
}

impl PgClient {
    pub async fn new() -> Result<Self> {
        let database_url = std::env::var("DATABASE_URL")?;
        let pool = PgPool::connect(&database_url).await?;
        Ok(Self { pool: pool })
    }

    pub async fn insert_embeddings(
        self: &Self,
        vectors: &Vec<Embedding>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        for embedding in vectors {
            let vector = Vector::from(embedding.vector.clone());

            sqlx::query("INSERT INTO vectors (uri, vector) VALUES ($1, $2)")
                .bind(&embedding.uri)
                .bind(vector)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
