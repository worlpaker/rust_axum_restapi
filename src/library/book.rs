use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Represents the status of a book.
#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Available,
    NOTAvailable,
    Rented,
}

impl Default for Status {
    fn default() -> Self {
        Self::Available
    }
}

/// Represents a book.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct Book {
    pub name: String,
    pub year: i32,
    pub category: String,
    pub status: Status,
    pub author: String,
}

/// Represents the query parameters for filtering books.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow, IntoParams)]
pub struct BookQuery {
    pub name: Option<String>,
    pub year: Option<i32>,
    pub category: Option<String>,
    pub status: Option<Status>,
    pub author: Option<String>,
}

/// Inserts a book into the database.
///
/// ## Arguments
///
/// * `pool`: The PostgreSQL database connection pool.
/// * `book`: The book to insert.
///
/// ## Returns
///
/// The UUID of the inserted book.
///
/// ## Errors
///
/// This function returns an error if the book insertion fails or if there
/// is an issue with the database connection.
pub async fn insert_book(pool: &PgPool, book: &Book) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO book (name, year, category, status, author)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING Id
        "#,
        book.name,
        book.year,
        book.category,
        book.status as Status,
        book.author,
    )
    .fetch_one(pool)
    .await
    .map(|record| record.id)
}

/// Retrieves a list of books from the database based on the provided query.
///
/// ## Arguments
///
/// * `pool`: The PostgreSQL database connection pool.
/// * `book`: The query parameters for filtering the books.
///
/// ## Returns
///
/// A vector of `Book` objects representing the retrieved books.
///
/// ## Errors
///
/// This function returns an error if the query fails or if there is an issue
/// with the database connection.
pub async fn books(pool: &PgPool, book: &BookQuery) -> Result<Vec<Book>, sqlx::Error> {
    let result = sqlx::query_as!(
        Book,
        r#"
        SELECT name, year, category, status as "status: _", author FROM book
        WHERE
            ($1::text IS NULL OR name = $1)
            AND ($2::integer IS NULL OR year = $2)
            AND ($3::text IS NULL OR category = $3)
            AND ($4::status IS NULL OR status = $4)
            AND ($5::text IS NULL OR author = $5)
        "#,
        book.name,
        book.year,
        book.category,
        book.status.unwrap_or_default() as Status,
        book.author,
    )
    .fetch_all(pool)
    .await?;
    if result.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }
    Ok(result)
}

/// Retrieves detailed information about a specific book from the database.
///
/// ## Arguments
///
/// * `pool`: The PostgreSQL database connection pool.
/// * `book_id`: The ID of the book to retrieve.
///
/// ## Returns
///
/// A `Book` object containing the detailed information of the retrieved book.
///
/// ## Errors
///
/// This function returns an error if the retrieval fails or if there is an issue
/// with the database connection.
pub async fn get_book(pool: &PgPool, book_id: Uuid) -> Result<Book, sqlx::Error> {
    sqlx::query_as!(
        Book,
        r#"
        SELECT name, year, category, status as "status: _", author
        FROM book
        WHERE id = $1
        "#,
        book_id,
    )
    .fetch_one(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::author::{self, Author};
    use fake::faker::lorem::en::Sentence as FakeTitle;
    use fake::faker::lorem::en::Word as FakeCategory;
    use fake::Fake;
    use rand::Rng;
    use std::ops::Range;

    impl Book {
        /// Creates a fake book object.
        ///
        /// This method generates a fake book with random attributes using the `fake` crate.
        /// It also inserts a fake author associated with the book into the database.
        ///
        /// ## Arguments
        ///
        /// * `pool`: The PostgreSQL database connection pool.
        ///
        /// ## Returns
        ///
        /// A `Book` object representing the fake book with randomly generated attributes.
        ///
        /// ## Panics
        ///
        /// This method will panic if it fails to insert the fake author into the database.
        pub async fn create_fake_book(pool: &PgPool) -> Self {
            // first insert an author
            let fake_author = Author::create_fake_author().await;
            let _insert_author = author::insert_author(pool, &fake_author)
                .await
                .expect("failed to insert fake author");
            let mut rng = rand::thread_rng();
            let fake_year = rng.gen_range(1900..=2023);
            Self {
                name: FakeTitle(Range { start: 10, end: 30 }).fake::<String>(),
                year: fake_year,
                category: FakeCategory().fake::<String>(),
                status: Status::default(),
                author: fake_author.name,
            }
        }
    }

    #[tokio::test]
    async fn test_book() {
        let pool = crate::database::postgres::init::pg_pool()
            .await
            .expect("failed to connect to postgres");
        let book = Book::create_fake_book(&pool).await;
        // insert_book
        let result_id = insert_book(&pool, &book)
            .await
            .expect("failed to insert book");
        let fetched_book = sqlx::query_as!(
            Book,
            r#"
            SELECT name, year, category, status as "status: _", author
            FROM book
            WHERE Id = $1
            "#,
            result_id,
        )
        .fetch_one(&pool)
        .await;
        assert_eq!(fetched_book.expect("unmatched book"), book);
        // get_book
        let get_book_result = get_book(&pool, result_id)
            .await
            .expect("failed to get book");
        assert_eq!(book.name, get_book_result.name);
        // books
        // 1: all books
        let books_result = books(
            &pool,
            &BookQuery {
                name: None,
                year: None,
                category: None,
                status: None,
                author: None,
            },
        )
        .await;
        assert!(books_result
            .as_ref()
            .map(|books| !books.is_empty())
            .unwrap_or_else(|_| false));
        assert!(books_result.is_ok());
        // 2: Get books by year (with year filter)
        let books_by_year_result = books(
            &pool,
            &BookQuery {
                name: None,
                year: Some(book.year),
                category: None,
                status: None,
                author: None,
            },
        )
        .await;
        assert!(books_by_year_result.is_ok());
        // 3: Get books by category (with category filter)
        let books_by_category_result = books(
            &pool,
            &BookQuery {
                name: None,
                year: None,
                category: Some(book.category.clone()),
                status: None,
                author: None,
            },
        )
        .await;
        assert!(books_by_category_result.is_ok());
        // 4: Get books by status (with status filter)
        let books_by_status_result = books(
            &pool,
            &BookQuery {
                name: None,
                year: None,
                category: None,
                status: Some(book.status),
                author: None,
            },
        )
        .await;
        assert!(books_by_status_result.is_ok());
        // 5: Get books by author (with author filter)
        let books_by_author_result = books(
            &pool,
            &BookQuery {
                name: None,
                year: None,
                category: None,
                status: None,
                author: Some(book.author.clone()),
            },
        )
        .await;
        assert!(books_by_author_result.is_ok());
        // 6: Exact given criteria
        let books_by_all_criteria = books(
            &pool,
            &BookQuery {
                name: Some(book.name.clone()),
                year: Some(book.year),
                category: Some(book.category.clone()),
                status: Some(book.status),
                author: Some(book.author.clone()),
            },
        )
        .await;
        assert!(books_by_all_criteria.is_ok());
    }
}
