# Cooper & Co. Website

Rust workspace for a Cooper & Co. pet-service website:

- `frontend`: Yew single-page app built with Trunk.
- `backend`: Rocket API and static-file server.
- SurrealDB-ready inquiry storage with a memory fallback when DB env vars are not configured.

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
- External SurrealDB instance reachable from backend, or use the built-in file-backed embedded database by leaving the remote credential env vars unset

## Backend Setup

1. Copy environment template:

   PowerShell:
## Public Facebook Details Used

The initial content was taken from the public Facebook page at `https://www.facebook.com/CooperAndCoPet`:

- Name: Cooper & Co.
- Category: Pet Service
- Location: Lorain County, OH
- Phone: `(440) 276-1716`
- Email: `cooper.copetservices@gmail.com`
- Yelp listing: `https://m.yelp.com/biz/cooper-and-company-elyria`
- Visible stats: 177 likes and 177 followers
- Visible update: summer group classes announcement from May 10

## Run Locally

Install the WASM target if needed:

```powershell
rustup target add wasm32-unknown-unknown
```

Build the frontend:

```powershell
cd frontend
trunk build
```

2. To use the built-in file-backed database, leave `SURREALDB_USERNAME` and `SURREALDB_PASSWORD` unset and keep `SURREALDB_PATH` pointed at a writable folder.
3. To use an external SurrealDB instance, populate all SurrealDB env vars in `backend/.env`.

## Run Backend
Run the backend:

```powershell
cd ../backend
cargo run
```

Open `http://127.0.0.1:9001`.

## Admin Login

The admin area is available at `/admin` and uses Microsoft OAuth. Create a Microsoft Entra app registration with this redirect URI:

```text
http://127.0.0.1:8080/auth/microsoft/callback
```

Set these environment variables before starting the backend:

```powershell
$env:MICROSOFT_CLIENT_ID="..."
$env:MICROSOFT_CLIENT_SECRET="..."
$env:MICROSOFT_TENANT_ID="common" # or your tenant ID
$env:MICROSOFT_REDIRECT_URI="http://127.0.0.1:8080/auth/microsoft/callback"
$env:ADMIN_ALLOWED_EMAILS="admin@example.com,second-admin@example.com"
```

Rocket private cookies require a stable production secret key. Set `ROCKET_SECRET_KEY` in deployment so existing admin sessions remain valid across restarts.

Admin APIs under `/api/admin/*` require either a valid Microsoft admin session cookie or an `Authorization: Bearer <token>` header matching `ADMIN_API_TOKEN`:

```powershell
$env:ADMIN_API_TOKEN="use-a-long-random-token"
```

## SEO Configuration

Rocket serves `robots.txt` and `sitemap.xml` explicitly. Set the public production URL before deployment so sitemap entries, canonical URLs, and social previews point at the live domain:

```powershell
$env:PUBLIC_SITE_URL="https://your-production-domain.example"
```

Production is indexable by default. For beta or staging deployments only, block crawlers with:

```powershell
$env:COOPERCO_NOINDEX="true"
```

## SurrealDB Configuration

When you have SurrealDB set up, provide these env vars before starting Rocket:

```powershell
trunk serve --config frontend/Trunk.toml
```

Frontend is available at `http://127.0.0.1:9000` and proxies `/api/*` to the backend on `http://127.0.0.1:9001`.

## Extend Next

- Add auth (JWT/session) and protect backend routes.
- Add POST/PUT/DELETE API routes for customer management.
- Add explicit schema/migration flow for SurrealDB records.
- Add deployment manifests (container, reverse proxy, hosted DB config).
