use crate::docs::api::ApiDoc;
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
pub mod author;
pub mod book;
pub mod user;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Represents a web application for a library.
///
/// This struct holds a reference to a PostgreSQL connection pool `PgPool` and
/// is used to handle web requests related to the library. It is cloneable and
/// exposes the `pool` field for accessing the connection pool.
#[derive(Clone)]
#[allow(dead_code)]
pub struct LibraryWeb {
    pool: PgPool,
}

impl LibraryWeb {
    /// Creates a new instance of `LibraryWeb`.
    ///
    /// This function takes a PostgreSQL connection pool `pool` and returns
    /// a new `LibraryWeb` instance.
    ///
    /// ## Arguments
    ///
    /// * `pool`: The PostgreSQL database connection pool.
    ///
    /// ## Returns
    ///
    /// A new `LibraryWeb` instance.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Sets up the router for the library web service.
    ///
    /// This function configures the router for handling various routes and
    /// middleware for tracing. It returns the configured `Router`.
    ///
    /// ## Returns
    ///
    /// A configured `Router` for the library web service.
    pub fn setup_router(self) -> Router {
        let book_routes = Router::new()
            .route("/", get(book::books))
            .route("/create", post(book::create_book))
            .route("/:book_id", get(book::get_book));

        let author_routes = Router::new()
            .route("/", get(author::authors))
            .route("/create", post(author::create_author))
            .route("/:author_id", get(author::get_author));

        let user_routes = Router::new()
            .route("/", get(user::users))
            .route("/create", post(user::create_user))
            .route("/rent/:nation_id", post(user::rent_book))
            .route("/:nation_id", get(user::get_user));

        Router::new()
            .nest("/api/book", book_routes)
            .nest("/api/author", author_routes)
            .nest("/api/user", user_routes)
            .merge(
                SwaggerUi::new("/api/swagger").url("/api/docs/openapi.json", ApiDoc::openapi()),
            )
            .layer(axum_tracing_opentelemetry::opentelemetry_tracing_layer())
            .with_state(self)
            .with_state(())
    }
}

#[cfg(test)]
pub mod tests {
    use axum::{
        body::Bytes,
        http::{header::CONTENT_TYPE, Method, Request},
    };
    use http_body::combinators::UnsyncBoxBody;
    use serde::{de::DeserializeOwned, Serialize};
    use tower::ServiceExt;

    use super::*;

    impl LibraryWeb {
        /// Creates a new instance of `LibraryWeb` for testing purposes.
        ///
        /// This function creates a new `LibraryWeb` instance with a PostgreSQL
        /// connection pool for testing. It returns the created instance.
        ///
        /// ## Returns
        ///
        /// A new `LibraryWeb` instance with a test PostgreSQL connection pool.
        pub async fn new_test() -> Self {
            Self {
                pool: crate::database::postgres::init::pg_pool()
                    .await
                    .expect("failed to create postgres pool"),
            }
        }
    }

    /// Sends a request to the specified router and returns the response.
    ///
    /// This function sends an HTTP request to the provided router and returns
    /// the corresponding HTTP response.
    ///
    /// ## Arguments
    ///
    /// * `router`: The router to send the request to.
    /// * `request`: The HTTP request to send.
    ///
    /// ## Returns
    ///
    /// The HTTP response returned by the router.
    pub async fn send_request(
        router: &Router,
        request: Request<hyper::Body>,
    ) -> hyper::Response<UnsyncBoxBody<Bytes, axum::Error>> {
        router
            .clone()
            .oneshot(request)
            .await
            .expect("failed to send oneshot request")
    }

    /// Sends a GET request to the specified router and returns the response.
    ///
    /// This function sends a GET request with the specified URI to the provided
    /// router and returns the corresponding HTTP response.
    ///
    /// ## Arguments
    ///
    /// * `router`: The router to send the request to.
    /// * `uri`: The URI for the GET request.
    ///
    /// ## Returns
    ///
    /// The HTTP response returned by the router.
    pub async fn get(
        router: &Router,
        uri: impl AsRef<str>,
    ) -> hyper::Response<UnsyncBoxBody<Bytes, axum::Error>> {
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri.as_ref())
            .body(hyper::Body::empty())
            .expect("failed to build GET request");
        send_request(router, request).await
    }

    /// Sends a POST request to the specified router and returns the response.
    ///
    /// This function sends a POST request with the specified URI and body to the
    /// provided router and returns the corresponding HTTP response.
    ///
    /// ## Arguments
    ///
    /// * `router`: The router to send the request to.
    /// * `uri`: The URI for the POST request.
    /// * `body`: The body of the POST request.
    ///
    /// ## Returns
    ///
    /// The HTTP response returned by the router.
    pub async fn post<T: Serialize>(
        router: &Router,
        uri: impl AsRef<str>,
        body: &T,
    ) -> hyper::Response<UnsyncBoxBody<Bytes, axum::Error>> {
        let request = Request::builder()
            .method(Method::POST)
            .uri(uri.as_ref())
            .header(CONTENT_TYPE, "application/json")
            .body(
                serde_json::to_vec(body)
                    .expect("failed to serialize POST body")
                    .into(),
            )
            .expect("failed to build POST request");
        send_request(router, request).await
    }

    /// Deserializes the response body into the specified type.
    ///
    /// This function takes an HTTP response and deserializes its body into the
    /// specified type `T`. It returns the deserialized value.
    ///
    /// ## Arguments
    ///
    /// * `response`: The HTTP response to deserialize.
    ///
    /// ## Returns
    ///
    /// The deserialized value of type `T`.
    ///
    /// ## Panics
    ///
    /// This function will panic if it fails to read the response body or if it
    /// fails to deserialize the response.
    pub async fn deserialize_response_body<T>(
        response: hyper::Response<UnsyncBoxBody<Bytes, axum::Error>>,
    ) -> T
    where
        T: DeserializeOwned,
    {
        let bytes = hyper::body::to_bytes(response.into_body())
            .await
            .expect("failed to read response body into bytes");
        serde_json::from_slice::<T>(&bytes).expect("failed to deserialize response")
    }
}
