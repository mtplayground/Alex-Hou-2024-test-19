# Alex-Hou-2024-test-19

Leptos full-stack TodoMVC foundation using Axum for SSR/hydration and PostgreSQL as the persistent datastore.

## Prerequisites

- Rust toolchain
- `cargo-leptos`
- PostgreSQL if running locally without Docker
- Docker and Docker Compose if using the container workflow

This repository is currently set up with the app shell, runtime configuration, and container workflow. The TodoMVC domain, migrations, and SQL access are added in later issues.

## Environment Variables

The server loads `.env` first and `.env.production` second, then reads the following required keys:

| Key | Required | Purpose | Example |
| --- | --- | --- | --- |
| `DATABASE_URL` | Yes | PostgreSQL connection string used by the server configuration | `postgresql://postgres:postgres@localhost:5432/todos?sslmode=disable` |
| `LEPTOS_SITE_ADDR` | Yes | Socket address the Axum server binds to | `0.0.0.0:8080` |
| `RUST_LOG` | Yes | `tracing` filter for structured JSON logs | `info,alex_hou_2024_test_19=debug` |

Bootstrap a local env file from the committed example:

```bash
cp .env.example .env
```

Then update `DATABASE_URL` for your environment if you are not using the default Docker Compose database.

## Local Development

1. Ensure PostgreSQL is available and `DATABASE_URL` points to it.
2. Create a local env file.
3. Start the Leptos development server.

```bash
cp .env.example .env
cargo leptos watch
```

The app binds to `http://127.0.0.1:8080` with the current default env values.

If you are working inside this Sprite environment, you can use the provisioned database URL directly:

```bash
export DATABASE_URL=$(cat /workspace/.database_url)
export LEPTOS_SITE_ADDR=0.0.0.0:8080
export RUST_LOG=info,alex_hou_2024_test_19=debug
cargo leptos watch
```

## Docker Workflow

The repository includes:

- `Dockerfile`: multi-stage build that runs `cargo leptos build --release`
- `docker-compose.yml`: local app + PostgreSQL stack for end-to-end runs

Start the stack with:

```bash
docker compose up --build
```

Endpoints and services:

- App: `http://127.0.0.1:8080`
- PostgreSQL: `localhost:5432`

Stop the stack:

```bash
docker compose down
```

Remove the local Postgres volume too:

```bash
docker compose down -v
```

The Compose app service injects its own `DATABASE_URL`, `LEPTOS_SITE_ADDR`, and `RUST_LOG`, so it does not rely on your local `.env` file.

## Architecture Overview

Current runtime layout:

```text
┌─────────────┐
│   Browser   │
└──────┬──────┘
       │ HTTP
       v
┌──────────────────────────────┐
│ Axum + Leptos server         │
│ src/main.rs                  │
│ - loads env in src/config.rs │
│ - initializes tracing        │
│ - mounts Leptos routes       │
└──────┬───────────────┬───────┘
       │               │
       │ SSR/Hydrate   │ static assets
       v               v
┌──────────────┐   ┌──────────────┐
│ src/app.rs   │   │ target/site  │
│ UI shell     │   │ cargo-leptos │
└──────────────┘   └──────────────┘
       │
       │ DATABASE_URL
       v
┌──────────────────────────────┐
│ PostgreSQL                   │
│ local Docker service today   │
│ sqlx wiring added next       │
└──────────────────────────────┘
```

Repository landmarks:

- `src/main.rs`: Axum server bootstrap
- `src/config.rs`: env loading and structured logging setup
- `src/app.rs`: root Leptos app shell
- `Dockerfile`: release container build
- `docker-compose.yml`: local app + database stack
