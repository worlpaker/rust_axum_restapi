use super::LibraryWeb;
use crate::helper::web::{internal_server_error, Response};
use crate::library::author::{self, Author, AuthorQuery, AuthorRow};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents the body of a request to create an author.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreatedAuthorBody {
    pub info: Author,
    pub id: Uuid,
}

/// Represents the body of a response containing multiple authors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AuthorsBody {
    pub authors: Vec<Author>,
}

/// Represents the body of a response containing a single author.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GetAuthorBody {
    pub author: AuthorRow,
}

#[utoipa::path(
    post,
    path = "/api/author/create",
    tag = "author",
    request_body = Author,
    responses(
        (status = 201, description = "author created succesfully", body = CreatedAuthorBody),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn create_author(
    State(library_web): State<LibraryWeb>,
    Json(author): Json<Author>,
) -> Response<CreatedAuthorBody> {
    let Ok(author_id) = author::insert_author(&library_web.pool, &author).await else {
        return internal_server_error().await;
    };
    let response = CreatedAuthorBody {
        info: author,
        id: author_id,
    };
    (StatusCode::CREATED, Ok(Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/author",
    tag = "author",
    params(
        AuthorQuery
    ),
    responses(
        (status = 200, description = "list matching authors", body = AuthorsBody),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn authors(
    State(library_web): State<LibraryWeb>,
    Query(author): Query<AuthorQuery>,
) -> Response<AuthorsBody> {
    let Ok(authors) = author::authors(&library_web.pool, &author).await else {
        return internal_server_error().await;
    };
    let response = AuthorsBody { authors };
    (StatusCode::OK, Ok(Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/author/{author_id}",
    tag = "author",
    params(
        ("author_id"= Uuid, Path,),
    ),
    responses(
        (status = 200, description = "list author", body = GetAuthorBody),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_author(
    State(library_web): State<LibraryWeb>,
    Path(author_id): Path<Uuid>,
) -> Response<GetAuthorBody> {
    let author = match author::get_author(&library_web.pool, author_id).await {
        Ok(author) => author,
        Err(_) => return internal_server_error().await,
    };
    let response = GetAuthorBody { author };
    (StatusCode::OK, Ok(Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library_web::tests::{deserialize_response_body, get, post};
    use urlencoding::encode;

    async fn concurrency_create_author(router: axum::Router, author: Author) -> StatusCode {
        let response = post(&router, "/api/author/create", &author).await;
        response.status()
    }

    #[tokio::test]
    async fn test_concurrency_create_author() {
        let router = LibraryWeb::new_test().await.setup_router();
        let request_body = Author::create_fake_author().await;
        let fut_a = concurrency_create_author(router.clone(), request_body.clone());
        let fut_b = concurrency_create_author(router.clone(), request_body.clone());
        let (status_a, status_b) = tokio::join!(fut_a, fut_b);
        assert_eq!(status_a.min(status_b), 201, "should succeed");
        assert_eq!(status_a.max(status_b), 500, "should fail");
    }

    #[tokio::test]
    async fn test_create_author_and_get_author() {
        let router = LibraryWeb::new_test().await.setup_router();
        let request_body = Author::create_fake_author().await;
        let response = post(&router, "/api/author/create", &request_body).await;
        assert_eq!(response.status(), 201);

        let response_body = deserialize_response_body::<CreatedAuthorBody>(response).await;
        assert_eq!(response_body.info, request_body);

        let uri = format!("/api/author/{}", response_body.id);
        let response = get(&router, uri).await;
        assert_eq!(response.status(), 200);

        let response_body = deserialize_response_body::<GetAuthorBody>(response).await;
        assert_eq!(response_body.author.name, request_body.name);
    }

    #[tokio::test]
    async fn test_authors() {
        let router = LibraryWeb::new_test().await.setup_router();
        let request_body = Author::create_fake_author().await;
        let response = post(&router, "/api/author/create", &request_body).await;
        assert_eq!(response.status(), 201);

        let response_body_created = deserialize_response_body::<CreatedAuthorBody>(response).await;
        assert_eq!(response_body_created.info, request_body);

        let response = get(&router, "/api/author").await;
        assert_eq!(response.status(), 200);
        let response_body = deserialize_response_body::<AuthorsBody>(response).await;
        assert!(response_body.authors.contains(&request_body));
        let uri = format!(
            "/api/author?name={}",
            encode(&response_body_created.info.name)
        );
        let response = get(&router, uri).await;
        assert_eq!(response.status(), 200);
        let response_body = deserialize_response_body::<AuthorsBody>(response).await;
        assert_eq!(response_body.authors[0].name, request_body.name);

        let uri = format!(
            "/api/author?name={}&country={}",
            encode(&response_body_created.info.name),
            encode(&response_body_created.info.country),
        );
        let response = get(&router, uri).await;
        assert_eq!(response.status(), 200);
        let response_body = deserialize_response_body::<AuthorsBody>(response).await;
        assert_eq!(response_body.authors[0].name, request_body.name);
        assert_eq!(response_body.authors[0].country, request_body.country);
    }
}
