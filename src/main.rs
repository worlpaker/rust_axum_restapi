mod database;
mod docs;
mod helper;
mod library;
mod library_web;
mod telemetry;

use crate::library_web::LibraryWeb;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    telemetry::init::tracing();

    let pool = database::postgres::init::pg_pool()
        .await
        .expect("failed to connect to postgres");
    let router = LibraryWeb::new(pool).setup_router();
    // if you run with local:
    // let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));

    tracing::info!("listening on http://{}", addr);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .expect("failed to serve");
}
