# Cooper & Co. Rust Starter

Minimal full-stack Rust workspace template:

- `frontend`: Yew app (served with Trunk)
- `backend`: Rocket API + backend-only SurrealDB access

The frontend calls HTTP endpoints on the backend. It never connects to SurrealDB directly.

## Project Structure

```text
.
├── Cargo.toml            # Workspace manifest
├── backend/
│   ├── .env.example
│   ├── Cargo.toml
│   └── src/
│       ├── config.rs
│       ├── db.rs
│       ├── main.rs
│       ├── models.rs
│       └── routes.rs
└── frontend/
    ├── Cargo.toml
    ├── Trunk.toml
    ├── index.html
    └── src/
        ├── api.rs
        └── main.rs
```

## Prerequisites

- Rust (stable)
- `trunk` (`cargo install trunk`)
- `wasm32` target (`rustup target add wasm32-unknown-unknown`)
- External SurrealDB instance reachable from backend

## Backend Setup

1. Copy environment template:

   PowerShell:

   ```powershell
   Copy-Item backend/.env.example backend/.env
   ```

2. Update `backend/.env` values for your external SurrealDB instance.

## Run Backend

```powershell
cargo run -p backend
```

Backend listens on `http://127.0.0.1:8000` and exposes:

- `GET /api/health`
- `GET /api/customers`

## Run Frontend

In a second terminal:

```powershell
trunk serve --config frontend/Trunk.toml
```

Frontend is available at `http://127.0.0.1:8080` and proxies `/api/*` to the backend.

## Extend Next

- Add auth (JWT/session) and protect backend routes.
- Add POST/PUT/DELETE API routes for customer management.
- Add explicit schema/migration flow for SurrealDB records.
- Add deployment manifests (container, reverse proxy, hosted DB config).

## Building

This website is built using Rust with WASM.