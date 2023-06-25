use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Represents an author.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Author {
    pub name: String,
    pub country: String,
    pub birth_date: String,
}

/// Represents a row in the author table of the database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct AuthorRow {
    pub name: String,
    pub country: String,
    pub birth_date: String,
    pub books: Option<Vec<String>>,
}

/// Represents a query for filtering authors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, IntoParams)]
pub struct AuthorQuery {
    pub name: Option<String>,
    pub country: Option<String>,
    pub birth_date: Option<String>,
}

/// Inserts an author into the database.
///
/// ## Arguments
///
/// * `pool`: The PostgreSQL database connection pool.
/// * `author`: The author to insert.
///
/// ## Returns
///
/// The UUID of the inserted author.
///
/// ## Errors
///
/// This function returns an error if the author insertion fails or
/// if there is an issue with the database connection.
pub async fn insert_author(pool: &PgPool, author: &Author) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO author (name, country, birth_date)
            VALUES ($1, $2, $3)
            RETURNING Id
        "#,
        author.name,
        author.country,
        author.birth_date,
    )
    .fetch_one(pool)
    .await
    .map(|record| record.id)
}

/// Retrieves a list of authors from the database based on the provided query.
///
/// ## Arguments
///
/// * `pool`: The PostgreSQL database connection pool.
/// * `author`: The author query parameters.
///
/// ## Returns
///
/// A vector of `Author` objects that match the query criteria.
///
/// ## Errors
///
/// This function returns an error if the query fails or if there is an issue
/// with the database connection.
pub async fn authors(pool: &PgPool, author: &AuthorQuery) -> Result<Vec<Author>, sqlx::Error> {
    let result = sqlx::query_as!(
        Author,
        r#"
        SELECT name, country, birth_date FROM author
        WHERE
            ($1::text IS NULL OR name = $1)
            AND ($2::text IS NULL OR country = $2)
            AND ($3::text IS NULL OR birth_date = $3)
        "#,
        author.name,
        author.country,
        author.birth_date,
    )
    .fetch_all(pool)
    .await?;
    if result.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }
    Ok(result)
}

/// Retrieves detailed information about a specific author from the database.
///
/// ## Arguments
///
/// * `pool`: The PostgreSQL database connection pool.
/// * `author_id`: The ID of the author to retrieve.
///
/// ## Returns
///
/// An `AuthorRow` object containing detailed information about the author.
///
/// ## Errors
///
/// This function returns an error if the retrieval fails or if there is an issue
/// with the database connection.
pub async fn get_author(pool: &PgPool, author_id: Uuid) -> Result<AuthorRow, sqlx::Error> {
    sqlx::query_as!(
        AuthorRow,
        r#"
        SELECT author.name,
            (SELECT array_agg(book.name) FROM book WHERE book.author = author.name) as books,
            author.birth_date,
            author.country
        FROM author
        WHERE author.id = $1;
        "#,
        author_id,
    )
    .fetch_one(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use fake::faker::address::en::CountryName;
    use fake::faker::name::en::Name as FakeName;
    use fake::Fake;
    use rand::Rng;

    impl Author {
        /// Generates a fake author with randomized information.
        ///
        /// ## Returns
        ///
        /// - `Self`: An `Author` object containing the fake author's information.
        pub async fn create_fake_author() -> Self {
            Self {
                name: FakeName().fake::<String>(),
                country: CountryName().fake::<String>(),
                birth_date: Self::create_fake_date().await,
            }
        }
        /// Creates a fake birth date within a range of 20 to 60 years ago.
        ///
        /// ## Returns
        ///
        /// - `String`: A formatted string representing the fake birth date in the format "YYYY-MM-DD".
        ///
        pub async fn create_fake_date() -> String {
            let mut rng = rand::thread_rng();
            let years = rng.gen_range(20..60);
            let birth_date = Utc::now() - Duration::days(years * 365);
            birth_date.format("%Y-%m-%d").to_string()
        }
    }

    #[tokio::test]
    async fn test_author() {
        let pool = crate::database::postgres::init::pg_pool()
            .await
            .expect("failed to connect to postgres");
        let author = Author::create_fake_author().await;
        // insert_author
        let result_id = insert_author(&pool, &author)
            .await
            .expect("failed to insert author");
        let fetched_author = sqlx::query_as!(
            Author,
            r#"
            SELECT name, country, birth_date FROM author WHERE Id = $1
            "#,
            result_id,
        )
        .fetch_one(&pool)
        .await;
        assert_eq!(fetched_author.expect("unmatched author"), author);
        // get_author
        let get_author_result = get_author(&pool, result_id)
            .await
            .expect("failed to get author");
        assert_eq!(author.name, get_author_result.name);
        // authors
        // 1: all authors
        let authors_result = authors(
            &pool,
            &AuthorQuery {
                name: None,
                country: None,
                birth_date: None,
            },
        )
        .await;
        assert!(authors_result
            .as_ref()
            .map(|authors| !authors.is_empty())
            .unwrap_or_else(|_| false));
        assert!(authors_result.is_ok());
        // 2: Get authors by country (with country filter)
        let authors_by_country_result = authors(
            &pool,
            &AuthorQuery {
                name: None,
                country: Some(author.country.clone()),
                birth_date: None,
            },
        )
        .await;
        assert!(authors_by_country_result.is_ok());
        // 3: Get authors by birth date (with birth date filter)
        let authors_by_birth_date_result = authors(
            &pool,
            &AuthorQuery {
                name: None,
                country: None,
                birth_date: Some(author.birth_date.clone()),
            },
        )
        .await;
        assert!(authors_by_birth_date_result.is_ok());
        // 4: exact given criterias
        let authors_by_all_criteria = authors(
            &pool,
            &AuthorQuery {
                name: Some(author.name.clone()),
                country: Some(author.country.clone()),
                birth_date: Some(author.birth_date.clone()),
            },
        )
        .await;
        assert!(authors_by_all_criteria.is_ok());
    }
}
