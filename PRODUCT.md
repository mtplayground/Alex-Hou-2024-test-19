# Product Snapshot

`alex-hou-2024-test-19` is a full-stack TodoMVC application built with Leptos and Axum, using PostgreSQL for all persistent state. It renders on the server, hydrates on the client, and stores todos in a real database rather than browser-only state.

# What It Does

- Creates, edits, toggles, filters, and deletes todos.
- Supports bulk actions: toggle all and clear completed.
- Persists todos in PostgreSQL with SQLx-backed queries and startup migrations.
- Serves the TodoMVC UI with SSR HTML, generated hydration assets, and vendored TodoMVC CSS.

# Architecture

- `src/main.rs` boots the Axum SSR server, loads env config, initializes structured `tracing`, creates the `PgPool`, runs embedded migrations, and provides the pool into Leptos context.
- `src/app.rs` owns the routed shell and maps `/`, `/active`, and `/completed` to the same TodoMVC page with route-derived filter state.
- `src/todo/model.rs` defines the shared `Todo` and `Filter` types used by both server and hydrate targets.
- `src/todo/repository.rs` is the PostgreSQL data layer.
- `src/todo/server.rs` exposes Leptos `#[server]` functions for reads and mutations.
- `src/todo/components/` contains the TodoMVC UI components and shared todo resource/state wiring.

# Conventions

- PostgreSQL is required. `DATABASE_URL`, `LEPTOS_SITE_ADDR`, and `RUST_LOG` are loaded from the environment.
- Migrations are embedded with `sqlx::migrate!()` and run automatically on server startup.
- Server-side data access goes through SQLx and the repository layer; no in-memory or file-based persistence is used.
- The TodoMVC CSS is vendored under `public/node_modules/...` and shipped as static assets.
- Local dev uses `cargo leptos watch`; containerized runs use the repo `Dockerfile` and `docker-compose.yml`.

# Current Quality Bar

- Repository integration tests run against PostgreSQL.
- Server function tests cover the main mutation and validation paths.
- The production container build copies migrations and static assets, and the compose workflow is documented for end-to-end local runs.
