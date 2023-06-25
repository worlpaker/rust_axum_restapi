use super::LibraryWeb;
use crate::helper::web::{internal_server_error, Response};
use crate::library::user::{
    self, RentBook, User, UserHistoryRow, UserQuery, UserRentBook, UserRow,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents the body of a response when a user created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreatedUserBody {
    pub info: User,
    pub id: Uuid,
}

/// Represents the body of a response containing multiple users.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UsersBody {
    pub users: Vec<UserRow>,
}

/// Represents the body of a response containing a user's history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct GetUserBody {
    pub user: Vec<UserHistoryRow>,
}

/// Represents the body of a response when a user rents a book.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RentedBookBody {
    pub message: String,
    pub info: UserRentBook,
}

#[utoipa::path(
    post,
    path = "/api/user/create",
    tag = "user",
    request_body = User,
    responses(
        (status = 201, description = "user created succesfully", body = CreatedUserBody),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn create_user(
    State(library_web): State<LibraryWeb>,
    Json(user): Json<User>,
) -> Response<CreatedUserBody> {
    let Ok(user_id) = user::insert_user(&library_web.pool, &user).await else {
        return internal_server_error().await;
    };
    let response = CreatedUserBody {
        info: user,
        id: user_id,
    };
    (StatusCode::CREATED, Ok(Json(response)))
}

#[utoipa::path(
    post,
    path = "/api/user/rent",
    tag = "user",
    request_body = RentBook,
    params(
        ("nation_id" = String, Path,),
    ),
    responses(
        (status = 201, description = "book rented succesfully", body = RentedBookBody),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn rent_book(
    State(library_web): State<LibraryWeb>,
    Path(nation_id): Path<String>,
    Json(book): Json<RentBook>,
) -> Response<RentedBookBody> {
    let info = UserRentBook {
        nation_id,
        book_name: book.book_name,
        due_date: book.due_date,
    };
    if user::rent_book(&library_web.pool, &info).await.is_err() {
        return internal_server_error().await;
    };
    let response = RentedBookBody {
        message: "successfully book rented".to_owned(),
        info,
    };
    (StatusCode::CREATED, Ok(Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/user",
    tag = "user",
    params(
        UserQuery
    ),
    responses(
        (status = 200, description = "list matching users", body = UsersBody),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn users(
    State(library_web): State<LibraryWeb>,
    Query(user): Query<UserQuery>,
) -> Response<UsersBody> {
    let Ok(users) = user::users(&library_web.pool, &user).await else {
        return internal_server_error().await;
    };
    let response = UsersBody { users };
    (StatusCode::OK, Ok(Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/user/{nation_id}",
    tag = "user",
    params(
        ("nation_id"= String, Path,),
    ),
    responses(
        (status = 200, description = "list user", body = GetUserBody),
        (status = 500, description = "Internal server error", body = String)
    )
)]
pub async fn get_user(
    State(library_web): State<LibraryWeb>,
    Path(nation_id): Path<String>,
) -> Response<GetUserBody> {
    let user = match user::get_user(&library_web.pool, nation_id).await {
        Ok(author) => author,
        Err(_) => return internal_server_error().await,
    };
    let response = GetUserBody { user };
    (StatusCode::OK, Ok(Json(response)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::book::{self, Book};
    use crate::library::user;
    use crate::library_web::tests::{deserialize_response_body, get, post};
    use urlencoding::encode;

    async fn concurrency_create_user(router: axum::Router, user: User) -> StatusCode {
        let response = post(&router, "/api/user/create", &user).await;
        response.status()
    }

    #[tokio::test]
    async fn test_concurrency_create_user() {
        let lib = LibraryWeb::new_test().await;
        let request_body = User::create_fake_user().await;
        let router = lib.setup_router();
        let fut_a = concurrency_create_user(router.clone(), request_body.clone());
        let fut_b = concurrency_create_user(router.clone(), request_body.clone());
        let (status_a, status_b) = tokio::join!(fut_a, fut_b);
        assert_eq!(status_a.min(status_b), 201, "should succeed");
        assert_eq!(status_a.max(status_b), 500, "should fail");
    }

    #[tokio::test]
    async fn test_create_user() {
        let lib = LibraryWeb::new_test().await;
        let request_body = User::create_fake_user().await;
        let router = lib.setup_router();

        let response = post(&router, "/api/user/create", &request_body).await;
        assert_eq!(response.status(), 201);
        let response_body = deserialize_response_body::<CreatedUserBody>(response).await;
        assert_eq!(response_body.info, request_body);
    }

    #[tokio::test]
    async fn test_rent_book_and_get_user() {
        let lib = LibraryWeb::new_test().await;

        // insert user, book
        let fake_user = User::create_fake_user().await;
        let _insert_fake_user = user::insert_user(&lib.pool, &fake_user)
            .await
            .expect("failed to insert fake user");
        let fake_book = Book::create_fake_book(&lib.pool).await;
        let _insert_fake_book = book::insert_book(&lib.pool, &fake_book)
            .await
            .expect("failed to insert fake book");

        // rent book
        let router = lib.setup_router();
        let uri = format!("/api/user/rent/{}?", encode(&fake_user.nation_id));
        let user_rent_book = RentBook {
            book_name: fake_book.name,
            due_date: "2023-05-09".to_owned(),
        };
        let response = post(&router, uri, &user_rent_book).await;
        assert_eq!(response.status(), 201);

        // get_user
        let uri = format!("/api/user/{}", &fake_user.nation_id);
        let response = get(&router, uri).await;
        assert_eq!(response.status(), 200);
        let response_body = deserialize_response_body::<GetUserBody>(response).await;
        assert_eq!(response_body.user[0].nation_id, fake_user.nation_id);
    }

    #[tokio::test]
    async fn test_users() {
        let lib = LibraryWeb::new_test().await;

        // insert user, book
        let fake_user = User::create_fake_user().await;
        let _insert_fake_user = user::insert_user(&lib.pool, &fake_user)
            .await
            .expect("failed to insert fake user");
        let fake_book = Book::create_fake_book(&lib.pool).await;
        let _insert_fake_book = book::insert_book(&lib.pool, &fake_book)
            .await
            .expect("failed to insert fake book");

        // rent book
        let router = lib.setup_router();
        let uri = format!("/api/user/rent/{}?", encode(&fake_user.nation_id));
        let user_rent_book = RentBook {
            book_name: fake_book.name.clone(),
            due_date: "2023-05-09".to_owned(),
        };
        let response = post(&router, uri, &user_rent_book).await;
        assert_eq!(response.status(), 201);

        // users
        let uri = format!("/api/user/{}", &fake_user.nation_id);
        let response = get(&router, uri).await;
        assert_eq!(response.status(), 200);
        let response_body = deserialize_response_body::<GetUserBody>(response).await;
        assert_eq!(response_body.user[0].nation_id, fake_user.nation_id);

        let uri = format!("/api/user?user_name={}", encode(&fake_user.name));
        let response = get(&router, uri).await;
        assert_eq!(response.status(), 200);
        let response_body = deserialize_response_body::<UsersBody>(response).await;
        assert_eq!(response_body.users[0].user_name, fake_user.name);

        let uri = format!("/api/user?book_name={}", encode(&fake_book.name),);
        let response = get(&router, uri).await;
        assert_eq!(response.status(), 200);
        let response_body = deserialize_response_body::<UsersBody>(response).await;
        assert_eq!(response_body.users[0].book_name, fake_book.name);

        let uri = format!(
            "/api/user?user_name={}&book_name={}",
            encode(&fake_user.name),
            encode(&fake_book.name),
        );
        let response = get(&router, uri).await;
        assert_eq!(response.status(), 200);
        let response_body = deserialize_response_body::<UsersBody>(response).await;
        assert_eq!(response_body.users[0].user_name, fake_user.name);
        assert_eq!(response_body.users[0].book_name, fake_book.name);
    }

    async fn concurrency_rent_book(router: axum::Router, user: User, book: Book) -> StatusCode {
        let uri = format!("/api/user/rent/{}?", encode(&user.nation_id));
        let user_rent_book = RentBook {
            book_name: book.name,
            due_date: "2023-05-09".to_owned(),
        };
        let response = post(&router, uri, &user_rent_book).await;
        response.status()
    }

    #[tokio::test]
    async fn test_concurrency_rent_book() {
        let lib = LibraryWeb::new_test().await;
        let fake_user_1 = User::create_fake_user().await;
        let _insert_fake_user_1 = user::insert_user(&lib.pool, &fake_user_1)
            .await
            .expect("failed to insert fake user");
        let fake_user_2 = User::create_fake_user().await;
        let _insert_fake_user_2 = user::insert_user(&lib.pool, &fake_user_2)
            .await
            .expect("failed to insert fake user");
        let fake_book = Book::create_fake_book(&lib.pool).await;
        let _insert_fake_book = book::insert_book(&lib.pool, &fake_book)
            .await
            .expect("failed to insert fake book");
        let router = lib.setup_router();
        let fut_a = concurrency_rent_book(router.clone(), fake_user_1, fake_book.clone());
        let fut_b = concurrency_rent_book(router.clone(), fake_user_2, fake_book.clone());
        let (status_a, status_b) = tokio::join!(fut_a, fut_b);
        assert_eq!(status_a.min(status_b), 201, "should succeed");
        assert_eq!(status_a.max(status_b), 500, "should fail");
    }
}
