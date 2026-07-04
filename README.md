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

The admin area is available at `/admin` and uses Microsoft Entra OAuth 2.0 Authorization Code flow with PKCE.

Current flow:

1. The Yew admin page renders a "Sign in with Microsoft" link to `/auth/microsoft/login`.
2. Rocket handles `/auth/microsoft/login`, generates `state`, `nonce`, a PKCE `code_verifier`, and a `code_challenge`.
3. Rocket stores the transient OAuth context in an encrypted, HttpOnly, SameSite=Lax private cookie and redirects to `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize`.
4. Microsoft redirects back to the backend callback route, `/auth/microsoft/callback`.
5. Rocket validates `state`, exchanges the code with the saved PKCE verifier, validates the ID token signature and claims, fetches the Microsoft profile, creates the admin session cookie, and redirects back to `/admin`.
6. The frontend does not infer auth from URL parameters. It calls `/api/admin/me` and relies on the backend session cookie.

### Microsoft Entra app registration

Create or update an app registration in Microsoft Entra ID:

- Platform type: Web
- Redirect URI for local backend testing:

```text
http://127.0.0.1:9001/auth/microsoft/callback
```

- Redirect URI for production: use the public backend URL that reaches Rocket, for example:

```text
https://api.example.com/auth/microsoft/callback
```

If the same public host serves both the frontend and backend, the production redirect can be:

```text
https://www.example.com/auth/microsoft/callback
```

The redirect URI must match `MICROSOFT_REDIRECT_URI` exactly, including scheme, host, port, and path. Do not register the Trunk dev server or a static frontend host for the callback unless that host proxies `/auth/microsoft/callback` to Rocket.

Required delegated Microsoft Graph scopes:

```text
openid profile email User.Read
```

Set these environment variables before starting Rocket locally:

```powershell
$env:MICROSOFT_CLIENT_ID="..."
$env:MICROSOFT_CLIENT_SECRET="..." # optional when using PKCE as a public client; recommended for confidential web apps
$env:MICROSOFT_TENANT_ID="common" # or your tenant ID
$env:BACKEND_BASE_URL="http://127.0.0.1:9001"
$env:PUBLIC_APP_URL="http://127.0.0.1:9001"
$env:MICROSOFT_REDIRECT_URI="http://127.0.0.1:9001/auth/microsoft/callback"
$env:MICROSOFT_POST_LOGIN_REDIRECT_URI="http://127.0.0.1:9001/admin"
$env:ADMIN_ALLOWED_EMAILS="admin@example.com,second-admin@example.com"
```

Cookie settings:

- OAuth state and admin session cookies are encrypted Rocket private cookies.
- Cookies are `HttpOnly`, `SameSite=Lax`, and `Path=/`.
- `Secure` is automatic when `PUBLIC_APP_URL` starts with `https://`; override with `COOKIE_SECURE=true` if needed.
- Set `COOKIE_DOMAIN` only in production when the cookie must span subdomains, for example `.example.com`.
- Rocket private cookies require a stable production secret key. Set `ROCKET_SECRET_KEY` in deployment so existing admin sessions remain valid across restarts.

For production behind a reverse proxy:

- `MICROSOFT_REDIRECT_URI` must be the public HTTPS callback URL, not the internal Rocket listener URL.
- `PUBLIC_APP_URL` must be the public frontend/app URL used after login.
- `BACKEND_BASE_URL` should be the public backend URL used to construct defaults.
- The proxy must route `/auth/*` and `/api/*` to Rocket before any SPA fallback. Otherwise the Yew/static frontend can swallow `/auth/microsoft/callback`.
- Forward the original host and scheme with headers such as `X-Forwarded-Host` and `X-Forwarded-Proto` according to your proxy/Rocket deployment setup.

OAuth failures are returned as visible HTTP errors and logged with `oauth event=...` records. Logs include state/nonce/PKCE fingerprints and redirect targets, but do not log client secrets, auth codes, tokens, refresh tokens, or raw cookies.

### Local OAuth test checklist

Run the local validation checks:

```powershell
cargo test -p backend
cargo check
```

Then start Rocket on `http://127.0.0.1:9001`, open `http://127.0.0.1:9001/admin`, and click "Sign in with Microsoft". The browser should leave the app for `login.microsoftonline.com`, return to `http://127.0.0.1:9001/auth/microsoft/callback`, and finish back at `http://127.0.0.1:9001/admin`. If it fails, check the Rocket logs for the `oauth event=...` entry that corresponds to the failed step.

Admin APIs under `/api/admin/*` require either a valid Microsoft admin session cookie or an `Authorization: Bearer <token>` header matching `ADMIN_API_TOKEN`:

```powershell
$env:ADMIN_API_TOKEN="use-a-long-random-token"
```

## SEO Configuration

Rocket serves `robots.txt` and `sitemap.xml` explicitly. Set the public production URL before deployment so sitemap entries, canonical URLs, and social previews point at the live domain:

```powershell
$env:PUBLIC_SITE_URLS="https://beta.cooper-and-co.com,https://cooper-and-co.com"
```

Production is indexable by default. For beta or staging deployments only, block crawlers with:

```powershell
$env:COOPERCO_NOINDEX="true"
```

The frontend build also copies static fallback files from `frontend/public/robots.txt` and `frontend/public/sitemap.xml` for deployments that serve `frontend/dist` directly.

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
