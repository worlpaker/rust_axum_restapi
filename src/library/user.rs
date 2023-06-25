use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{ToSchema, IntoParams};
use uuid::Uuid;

/// Represents a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct User {
    pub nation_id: String,
    pub name: String,
}

/// Represents a book rental by a user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct UserRentBook {
    pub nation_id: String,
    pub book_name: String,
    pub due_date: String,
}

/// Represents a row in the user's rental history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct UserHistoryRow {
    pub name: String,
    pub nation_id: String,
    pub book_name: String,
    pub due_date: String,
}

/// Represents a query for retrieving users.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow, IntoParams)]
pub struct UserQuery {
    pub user_name: Option<String>,
    pub book_name: Option<String>,
}

/// Represents a row in the user table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow, ToSchema)]
pub struct UserRow {
    pub nation_id: String,
    pub user_name: String,
    pub book_name: String,
}

/// Represents a book to be rented.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RentBook {
    pub book_name: String,
    pub due_date: String,
}

/// Inserts a new user into the database.
///
/// ## Arguments
///
/// * `pool`: The PostgreSQL database connection pool.
/// * `user`: The user to insert.
///
/// ## Returns
///
/// The UUID of the inserted user.
///
/// ## Errors
///
/// This function returns an error if the user insertion fails or
/// if there is an issue with the database connection.
pub async fn insert_user(pool: &PgPool, user: &User) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO users (nation_id, name)
            VALUES ($1, $2)
            RETURNING id
        "#,
        user.nation_id,
        user.name,
    )
    .fetch_one(pool)
    .await
    .map(|record| record.id)
}

/// Rent a book for a user.
///
/// ## Arguments
///
/// * `pool`: The PostgreSQL database connection pool.
/// * `data`: The rental information.
///
/// ## Returns
///
/// This function returns `Result<(), sqlx::Error>` indicating whether
/// the book rental was successful or if an error occurred.
///
/// ## Errors
///
/// This function returns an error if the book rental fails or
/// if there is an issue with the database connection.
pub async fn rent_book(pool: &PgPool, data: &UserRentBook) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;

    // Check if the book is available and update its status to 'Rented' in the same query
    let rent_book = sqlx::query!(
        r#"
        WITH updated_book AS (
            UPDATE book
            SET status = 'Rented'
            WHERE name = $1 AND status = 'Available'
            RETURNING name
        )
        INSERT INTO users_history (nation_id, book_name, due_date)
        SELECT $2, name, $3
        FROM updated_book
        "#,
        data.book_name,
        data.nation_id,
        data.due_date,
    )
    .execute(&mut transaction)
    .await?;

    if rent_book.rows_affected() == 0 {
        transaction.rollback().await?;
        return Err(sqlx::Error::RowNotFound);
    }

    transaction.commit().await
}

/// Retrieve a list of users based on the given query parameters.
///
/// ## Arguments
///
/// * `pool`: The PostgreSQL database connection pool.
/// * `user`: The query parameters for filtering users.
///
/// ## Returns
///
/// A vector of `UserRow` representing the retrieved user records.
///
/// ## Errors
///
/// This function returns an error if the user retrieval fails or
/// if there is an issue with the database connection.
pub async fn users(pool: &PgPool, user: &UserQuery) -> Result<Vec<UserRow>, sqlx::Error> {
    let result = sqlx::query_as!(
        UserRow,
        r#"
        SELECT users_history.book_name, users.nation_id, users.name as user_name
        FROM users_history
        JOIN users ON users_history.nation_id = users.nation_id
        WHERE
            ($1::text IS NULL OR users.name = $1)
            AND ($2::text IS NULL OR users_history.book_name = $2)
        "#,
        user.user_name,
        user.book_name
    )
    .fetch_all(pool)
    .await?;
    if result.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }
    Ok(result)
}

/// Retrieve the rental history of a user based on the given national ID.
///
/// ## Arguments
///
/// * `pool`: The PostgreSQL database connection pool.
/// * `nation_id`: The national ID of the user.
///
/// ## Returns
///
/// A vector of `UserHistoryRow` representing the user's rental history.
///
/// ## Errors
///
/// This function returns an error if the retrieval fails or
/// if there is an issue with the database connection.
pub async fn get_user(
    pool: &PgPool,
    nation_id: String,
) -> Result<Vec<UserHistoryRow>, sqlx::Error> {
    let result = sqlx::query_as!(
        UserHistoryRow,
        r#"
        SELECT users.name, users_history.nation_id, users_history.book_name, users_history.due_date
        FROM users_history
        JOIN users ON users.nation_id = users_history.nation_id 
        WHERE users_history.nation_id = $1
        "#,
        nation_id
    )
    .fetch_all(pool)
    .await?;
    if result.is_empty() {
        return Err(sqlx::Error::RowNotFound);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::author::Author;
    use crate::library::book::{self, Book};
    use fake::faker::name::en::Name as FakeUser;
    use fake::Fake;
    use rand::Rng;

    impl User {
        /// Creates a fake user with randomized national ID and name.
        ///
        /// ## Returns
        ///
        /// A `User` struct representing the fake user.
        ///
        /// ## Errors
        ///
        /// This function does not return any errors.
        pub async fn create_fake_user() -> Self {
            let mut rng = rand::thread_rng();
            Self {
                nation_id: rng.gen_range(10000000000_i64..=99999999999_i64).to_string(),
                name: FakeUser().fake::<String>(),
            }
        }
    }

    #[tokio::test]
    async fn test_user() {
        let pool = crate::database::postgres::init::pg_pool()
            .await
            .expect("failed to connect to postgres");
        // insert user
        let user = User::create_fake_user().await;
        let result_id = insert_user(&pool, &user)
            .await
            .expect("failed to insert user");
        let fetched_user = sqlx::query_as!(
            User,
            r#"
            SELECT nation_id, name
            FROM users
            WHERE id = $1
            "#,
            result_id,
        )
        .fetch_one(&pool)
        .await
        .expect("failed to fetch the user");
        assert_eq!(fetched_user, user);
        // insert book
        let book = Book::create_fake_book(&pool).await;
        let _insert_book = book::insert_book(&pool, &book)
            .await
            .expect("failed to insert book");
        let fake_delivery_date = Author::create_fake_date().await;
        let user_rent_book = &UserRentBook {
            nation_id: fetched_user.nation_id,
            book_name: book.name.clone(),
            due_date: fake_delivery_date,
        };
        // rent_book
        let _rent_result = rent_book(&pool, user_rent_book)
            .await
            .expect("failed to rent book");
        // get_user
        let get_user_result = get_user(&pool, user.nation_id.clone())
            .await
            .expect("failed to get user");
        assert_eq!(user.name, get_user_result[0].name);
        // users
        // 1: all criterias
        let user_query = &UserQuery {
            user_name: Some(user.name.clone()),
            book_name: Some(book.name.clone()),
        };
        let users_result = users(&pool, user_query).await.expect("failed to get users");
        assert!(users_result
            .iter()
            .any(|result| result.user_name == user.name.clone()
                && result.book_name == book.name.clone()));
        // 2: only user_name
        let user_query = &UserQuery {
            user_name: Some(user.name.clone()),
            book_name: None,
        };
        let users_result = users(&pool, user_query).await.expect("failed to get users");
        assert!(users_result
            .iter()
            .any(|result| result.user_name == user.name.clone()));
        // 3: only book_name
        let user_query = &UserQuery {
            user_name: None,
            book_name: Some(book.name.clone()),
        };
        let users_result = users(&pool, user_query).await.expect("failed to get users");
        assert!(users_result
            .iter()
            .any(|result| result.book_name == book.name.clone()));
        // 3: nothing is given
        let user_query = &UserQuery {
            user_name: None,
            book_name: None,
        };
        let users_result = users(&pool, user_query).await.expect("failed to get users");
        assert!(users_result
            .iter()
            .any(|result| result.user_name == user.name.clone()
                && result.book_name == book.name.clone()));
        // get_user
        let user_history_result = get_user(&pool, user.nation_id.clone())
            .await
            .expect("failed to get user");
        assert!(user_history_result
            .iter()
            .any(|result| result.nation_id == user.nation_id && result.book_name == book.name));
    }
}
