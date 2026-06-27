# Cooper & Co. Website

Rust workspace for a Cooper & Co. pet-service website:

- `frontend`: Yew single-page app built with Trunk.
- `backend`: Rocket API and static-file server.
- SurrealDB-ready inquiry storage with a memory fallback when DB env vars are not configured.

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

Run the backend:

```powershell
cd ../backend
cargo run
```

Open `http://127.0.0.1:8080`.

## SurrealDB Configuration

When you have SurrealDB set up, provide these env vars before starting Rocket:

```powershell
$env:SURREALDB_URL = "127.0.0.1:8000"
$env:SURREALDB_NS = "cooperco"
$env:SURREALDB_DB = "website"
$env:SURREALDB_USER = "root"
$env:SURREALDB_PASS = "root"
cargo run -p backend
```

Without `SURREALDB_URL`, inquiries are stored in memory for the current process.
