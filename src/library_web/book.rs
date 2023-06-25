use super::LibraryWeb;
use crate::helper::web::{internal_server_error, Response};
use crate::library::book::{self, Book, BookQuery};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents the body of a request to create a book.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreatedBookBody {
    pub info: Book,
    pub id: Uuid,
}

/// Represents the body of a response containing multiple books.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct BooksBody {
    pub books: Vec<Book>,
}

/// Represents the body of a response containing a single book.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GetBookBody {
    pub book: Book,
}

#[utoipa::path(
    post,
    path = "/api/book/create",
    tag = "book",
    request_body = Book,
    responses(
        (status = 201, description = "book created succesfully", body = CreatedBookBody),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn create_book(
    State(library_web): State<LibraryWeb>,
    Json(book): Json<Book>,
) -> Response<CreatedBookBody> {
    let Ok(book_id) = book::insert_book(&library_web.pool, &book).await else {
        return internal_server_error().await;
    };
    let response = CreatedBookBody {
        info: book,
        id: book_id,
    };
    (StatusCode::CREATED, Ok(Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/book",
    tag = "book",
    params(
        BookQuery
    ),
    responses(
        (status = 200, description = "list matching books", body = BooksBody),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn books(
    State(library_web): State<LibraryWeb>,
    Query(book): Query<BookQuery>,
) -> Response<BooksBody> {
    let Ok(books) = book::books(&library_web.pool, &book).await else {
        return internal_server_error().await;
    };
    let response = BooksBody { books };
    (StatusCode::OK, Ok(Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/book/{book_id}",
    tag = "book",
    params(
        ("book_id"= Uuid, Path,),
    ),
    responses(
        (status = 200, description = "list book", body = GetBookBody),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_book(
    State(library_web): State<LibraryWeb>,
    Path(book_id): Path<Uuid>,
) -> Response<GetBookBody> {
    let book = match book::get_book(&library_web.pool, book_id).await {
        Ok(book) => book,
        Err(_) => return internal_server_error().await,
    };
    let response = GetBookBody { book };
    (StatusCode::OK, Ok(Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library_web::tests::{deserialize_response_body, get, post};
    use urlencoding::encode;

    async fn concurrency_create_book(router: axum::Router, book: Book) -> StatusCode {
        let response = post(&router, "/api/book/create", &book).await;
        response.status()
    }

    #[tokio::test]
    async fn test_concurrency_create_book() {
        let lib = LibraryWeb::new_test().await;
        let request_body = Book::create_fake_book(&lib.pool).await;
        let router = lib.setup_router();
        let fut_a = concurrency_create_book(router.clone(), request_body.clone());
        let fut_b = concurrency_create_book(router.clone(), request_body.clone());
        let (status_a, status_b) = tokio::join!(fut_a, fut_b);
        assert_eq!(status_a.min(status_b), 201, "should succeed");
        assert_eq!(status_a.max(status_b), 500, "should fail");
    }

    #[tokio::test]
    async fn test_create_book_and_get_book() {
        let lib = LibraryWeb::new_test().await;
        let request_body = Book::create_fake_book(&lib.pool).await;
        let router = lib.setup_router();
        let response = post(&router, "/api/book/create", &request_body).await;
        assert_eq!(response.status(), 201);

        let response_body = deserialize_response_body::<CreatedBookBody>(response).await;
        assert_eq!(response_body.info, request_body);

        let uri = format!("/api/book/{}", response_body.id);
        let response = get(&router, uri).await;
        assert_eq!(response.status(), 200);

        let response_body = deserialize_response_body::<GetBookBody>(response).await;
        assert_eq!(response_body.book.name, request_body.name);
    }

    #[tokio::test]
    async fn test_books() {
        let lib = LibraryWeb::new_test().await;
        let request_body = Book::create_fake_book(&lib.pool).await;
        let router = lib.setup_router();

        let response = post(&router, "/api/book/create", &request_body).await;
        assert_eq!(response.status(), 201);
        let response_body_created = deserialize_response_body::<CreatedBookBody>(response).await;
        assert_eq!(response_body_created.info, request_body);

        let response = get(&router, "/api/book").await;
        assert_eq!(response.status(), 200);
        let response_body = deserialize_response_body::<BooksBody>(response).await;
        assert!(response_body.books.contains(&request_body));

        let uri = format!(
            "/api/book?name={}",
            encode(&response_body_created.info.name)
        );
        let response = get(&router, uri).await;
        assert_eq!(response.status(), 200);
        let response_body = deserialize_response_body::<BooksBody>(response).await;
        assert_eq!(response_body.books[0].name, request_body.name);

        let uri = format!(
            "/api/book?name={}&author={}",
            encode(&response_body_created.info.name),
            encode(&response_body_created.info.author),
        );
        let response = get(&router, uri).await;
        assert_eq!(response.status(), 200);
        let response_body = deserialize_response_body::<BooksBody>(response).await;
        assert_eq!(response_body.books[0].name, request_body.name);
        assert_eq!(response_body.books[0].author, request_body.author);
    }
}
