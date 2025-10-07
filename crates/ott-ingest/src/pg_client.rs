use sqlx::PgPool;

struct PgClient {
    pool: PgPool,
}
