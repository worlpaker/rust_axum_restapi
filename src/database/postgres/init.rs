use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env::var, time::Duration};

/// Creates a PostgreSQL connection pool.
///
/// This asynchronous function initializes a PostgreSQL connection pool using the
/// configuration specified in the environment variables. It returns a `Result`
/// with the `PgPool` if the pool is successfully created, or a `sqlx::Error` if
/// an error occurs during the process.
///
/// ## Returns
///
/// A `Result` containing the PostgreSQL connection pool (`PgPool`) or a `sqlx::Error`
/// if the pool creation fails.
///
/// ## Panics
///
/// This function will panic if it fails to load the `.env` file or if the `DATABASE_URL`
/// environment variable is not set.
pub async fn pg_pool() -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(1))
        .connect(&var("DATABASE_URL").expect("DATABASE_URL must be in environment"))
        .await
}
