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

   ```powershell
   Copy-Item backend/.env.example backend/.env
   ```

   The backend loads `.env` on startup. Values in `backend/.env` are the primary source. Shell environment variables are used only when a key is missing from `.env`.

2. To use the built-in memory fallback, leave `SURREALDB_URL` unset or empty.
3. To use an external SurrealDB instance, populate all SurrealDB env vars in `backend/.env`.

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

## Run Backend
Run the backend:

```powershell
cd ../backend
cargo run
```

Open `http://127.0.0.1:9001`.

Rocket serves the full application in this mode. It serves compiled frontend files from `frontend/dist` when run from the workspace root, or `../frontend/dist` when run from `backend/`. Set `COOPERCO_STATIC_DIR` if the build output is deployed somewhere else.

## Production Build And Run

Production does not require Trunk to be running. Cloudflare Tunnel should point directly at Rocket on port `9001`.

Build everything:

```powershell
trunk build --release --config frontend/Trunk.toml
cargo build -p backend --release
```

Or use the Makefile:

```powershell
make build-production
```

Run with the Makefile:

```powershell
make run-production
```

Run Rocket serving the compiled frontend and backend routes:

```powershell
.\target\release\backend.exe
```

On Linux:

```sh
./target/release/backend
```

Rocket route behavior in production:

- `/` and public marketing routes return route-specific crawlable HTML from Rocket.
- Static frontend assets are served by Rocket with normal file MIME types.
- `/admin` returns a noindex admin shell.
- `/api/*` and `/auth/*` are handled by Rocket backend routes and do not fall back to the frontend.

Cloudflare Tunnel ingress example:

```yaml
ingress:
  - hostname: cooper-and-co.com
    service: http://127.0.0.1:9001
  - service: http_status:404
```

Production `.env` values for `backend/.env`:

```dotenv
ROCKET_ADDRESS=127.0.0.1
ROCKET_PORT=9001
PUBLIC_APP_URL=https://cooper-and-co.com
BACKEND_BASE_URL=https://cooper-and-co.com
PRODUCTION_SITE_URL=https://cooper-and-co.com
MICROSOFT_REDIRECT_URI=https://cooper-and-co.com/auth/microsoft/callback
MICROSOFT_POST_LOGIN_REDIRECT_URI=https://cooper-and-co.com/admin
BEHIND_PROXY=true
COOKIE_SECURE=true
COOKIE_DOMAIN=cooper-and-co.com
ROCKET_SECRET_KEY=replace-with-a-stable-production-secret
```

For beta or staging, set `COOPERCO_NOINDEX=true` and use the staging host only for `PUBLIC_APP_URL`, `BACKEND_BASE_URL`, and Microsoft redirect settings.

Generate `ROCKET_SECRET_KEY` with a secure random 32-byte base64 value. Keep it stable across restarts so Rocket private cookies remain decryptable.

## Admin Login

The admin area is available at `/admin` and uses Microsoft Entra OAuth 2.0 Authorization Code flow with PKCE.

Current flow:

1. The Yew admin page renders a "Sign in with Microsoft" link to `/auth/microsoft/login`.
2. Rocket handles `/auth/microsoft/login`, generates `state`, `nonce`, a PKCE `code_verifier`, and a `code_challenge`.
3. Rocket stores the transient OAuth context in an encrypted, HttpOnly, SameSite=Lax private cookie and redirects to `https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize`.
4. Microsoft redirects back to `/auth/microsoft/callback`; Rocket handles this directly in production, and Trunk proxies `/auth/*` to Rocket only during local frontend development.
5. Rocket validates `state`, exchanges the code with the saved PKCE verifier, validates the ID token signature and claims, fetches the Microsoft profile, creates the admin session cookie, and redirects back to `/admin`.
6. The frontend does not infer auth from URL parameters. It calls `/api/admin/me` and relies on the backend session cookie.

### Microsoft Entra app registration

Create or update an app registration in Microsoft Entra ID:

- Platform type: Web
- Redirect URI for local frontend development with `trunk serve`:

```text
http://127.0.0.1:9000/auth/microsoft/callback
```

- Redirect URI for direct backend testing without Trunk:

```text
http://127.0.0.1:9001/auth/microsoft/callback
```

- Redirect URI for production with Cloudflare Tunnel pointed at Rocket:

```text
https://cooper-and-co.com/auth/microsoft/callback
```

If the same public host serves both the frontend and backend, the production redirect can be:

```text
https://www.example.com/auth/microsoft/callback
```

The redirect URI must match `MICROSOFT_REDIRECT_URI` exactly, including scheme, host, port, and path. In production, do not register the Trunk dev server; the tunnel should route `/auth/microsoft/callback` directly to Rocket.

Required delegated Microsoft Graph scopes:

```text
openid profile email User.Read
```

Set these values in `backend/.env` before starting Rocket locally. Shell environment variables are only used for keys that are not present in `backend/.env`:

```dotenv
MICROSOFT_CLIENT_ID=...
MICROSOFT_CLIENT_SECRET=... # optional when using PKCE as a public client; recommended for confidential web apps
MICROSOFT_TENANT_ID=common # or your tenant ID
BACKEND_BASE_URL=http://127.0.0.1:9000
PUBLIC_APP_URL=http://127.0.0.1:9000
MICROSOFT_REDIRECT_URI=http://127.0.0.1:9000/auth/microsoft/callback
MICROSOFT_POST_LOGIN_REDIRECT_URI=http://127.0.0.1:9000/admin
ADMIN_ALLOWED_EMAILS=admin@example.com,second-admin@example.com
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
- Cloudflare Tunnel should route the full hostname to Rocket on `http://127.0.0.1:9001`; no Trunk proxy is needed in production.
- Forward the original host and scheme with headers such as `X-Forwarded-Host` and `X-Forwarded-Proto` according to your proxy/Rocket deployment setup.

OAuth failures are returned as visible HTTP errors and logged with `oauth event=...` records. Logs include state/nonce/PKCE fingerprints and redirect targets, but do not log client secrets, auth codes, tokens, refresh tokens, or raw cookies.

### Local OAuth test checklist

Run the local validation checks:

```powershell
cargo test -p backend
cargo check
```

Then start Rocket on `http://127.0.0.1:9001` and Trunk on `http://127.0.0.1:9000`. Open `http://127.0.0.1:9000/admin` and click "Sign in with Microsoft". The browser should leave the app for `login.microsoftonline.com`, return to `http://127.0.0.1:9000/auth/microsoft/callback`, pass through the Trunk `/auth/*` proxy to Rocket, and finish back at `http://127.0.0.1:9000/admin`. If it fails, check the Rocket logs for the `oauth event=...` entry that corresponds to the failed step.

Production manual checks after `trunk build --release --config frontend/Trunk.toml` and starting Rocket on `9001`:

```sh
curl -vk http://127.0.0.1:9001/
curl -vk http://127.0.0.1:9001/auth/microsoft/login
curl -vk https://cooper-and-co.com/auth/microsoft/login
```

Expected results:

- `http://127.0.0.1:9001/` returns the frontend HTML.
- Both `/auth/microsoft/login` requests return a redirect to `login.microsoftonline.com`.
- The Microsoft authorization URL contains `redirect_uri=https%3A%2F%2Fcooper-and-co.com%2Fauth%2Fmicrosoft%2Fcallback`.

Admin APIs under `/api/admin/*` require either a valid Microsoft admin session cookie or an `Authorization: Bearer <token>` header matching `ADMIN_API_TOKEN`:

```dotenv
ADMIN_API_TOKEN=use-a-long-random-token
```

## SEO Configuration

Rocket serves route-specific crawlable HTML, `robots.txt`, and `sitemap.xml` from the centralized registry in `backend/src/seo.rs`. Production canonicals and sitemap entries point to:

```powershell
$env:PRODUCTION_SITE_URL="https://cooper-and-co.com"
```

Production is indexable by default. For beta or staging deployments only, block crawlers with:

```powershell
$env:COOPERCO_NOINDEX="true"
```

The frontend build also copies static fallback files from `frontend/public/robots.txt` and `frontend/public/sitemap.xml` for deployments that serve `frontend/dist` directly.

### Local SEO page foundation

The current SEO foundation includes confirmed public pages for:

- Services: dog training, puppy training, and group dog classes.
- Service areas: Lorain, Ohio. Nearby Lorain County communities are stored as unconfirmed candidates and are not included as indexable location pages.
- Resources: practical dog training, puppy training, and group-class articles.

Owner-confirmation fields are tracked in `content/business_profile.toml` and `docs/CONTENT_REQUIREMENTS.md`. Do not publish additional services, locations, prices, hours, testimonials, credentials, or policy claims until the owner confirms them.

### Search Console and analytics hooks

Set these before starting Rocket to emit verification/measurement hooks only when values are present and valid:

```powershell
$env:GOOGLE_SITE_VERIFICATION="google-site-verification-token"
$env:BING_SITE_VERIFICATION="bing-verification-token"
$env:GA4_MEASUREMENT_ID="G-XXXXXXXXXX"
```

Local and staging builds can omit analytics and verification IDs.

### SEO validation commands

Run these before SEO releases:

```powershell
cargo fmt
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
trunk build --config frontend/Trunk.toml
```

## SurrealDB Configuration

When you have SurrealDB set up, provide the SurrealDB values in `backend/.env` before starting Rocket.

```powershell
trunk serve --config frontend/Trunk.toml
```

For development, Trunk is available at `http://127.0.0.1:9000` and proxies `/api/*` and `/auth/*` to the backend on `http://127.0.0.1:9001`. This proxy is dev-only; production should run only Rocket on `9001`.

## Extend Next

- Add auth (JWT/session) and protect backend routes.
- Add POST/PUT/DELETE API routes for customer management.
- Add explicit schema/migration flow for SurrealDB records.
- Add deployment manifests (container, reverse proxy, hosted DB config).
