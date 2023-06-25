# Library API

Simple Rust Axum Rest API example for a library system. It provides functionalities for users to rent books and more.

## Features

- Framework: [Axum](https://github.com/tokio-rs/axum)

- Database: [PostgreSQL](https://www.postgresql.org/)

- Environment: [Docker](https://www.docker.com/)

- Tracing: [OpenTelemetry](https://github.com/open-telemetry/opentelemetry-rust)

- Monitoring: [Jaeger](https://github.com/jaegertracing/jaeger)

## Includes

- Swagger Documentation: It incorporates [utoipa](https://github.com/juhaku/utoipa) to generate comprehensive API documentation.

- Many-to-many RDBMS: The API efficiently handles complex relationships in the relational database management system.

- Asynchronous Concurrency: It utilizes async concurrency techniques to ensure smooth and reliable database transactions.

- Integration Testing: The API includes thorough integration tests covering both API endpoints and SQL functionality.

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/)

## Setup

- Clone the repository:

```sh
git clone https://github.com/worlpaker/rust_axum_restapi.git
```

- Make sure you are in the correct directory:

```sh
cd rust_axum_restapi
```

- Before starting the services, ensure that you set the necessary environment variables in the `docker-compose.yml`.

- Build containers and start services:

```sh
docker-compose up --build -d
```

- Detailed commands: [Docker Commands](https://docs.docker.com/engine/reference/commandline/docker/)

## Access

Backend: <http://localhost:8000/>

## API Endpoints

Information about each endpoint, including request/response formats and parameters, is available in the Swagger API documentation.

- Access Docs on API: <http://localhost:8000/api/swagger/>

- Alternatively, you can also access it manually at: `src/docs`

## Running Tests

- Run tests using the `cargo` command inside Docker container:

```sh
cargo test
```
