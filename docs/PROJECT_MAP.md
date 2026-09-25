# Project Map

Read this before changing anything. It says where each concern lives, so an
update starts in the right file instead of a repository-wide search.

Cooper & Co. is a dog-training and pet-service business in **Lorain County,
Ohio**. The site advertises primarily to Lorain County's cities;
see [SERVICE_AREAS.md](SERVICE_AREAS.md).

## How a request is served

```text
Browser ──► Cloudflare Tunnel ──► Rocket (backend, port 9001)
                                   ├─ /, /services/*, /resources/*, ... → seo.rs renders full HTML
                                   ├─ /robots.txt, /sitemap.xml          → seo.rs generates them
                                   ├─ /api/*, /auth/*                    → main.rs handlers
                                   ├─ static files (*.css, *.js, /assets) → frontend/dist
                                   └─ /admin and anything unmatched     → noindex Yew shell / 404
```

Public marketing pages are **server-rendered by Rocket** from
`backend/src/seo.rs`. The Yew app in `frontend/` is progressive enhancement
plus the admin UI; search engines never need it.

## Where to look

| To change... | Edit | Also update |
|---|---|---|
| Cities the site advertises to | `SERVICE_AREAS` and `lorain_county_cities!()` in `backend/src/seo.rs` | `LORAIN_COUNTY_CITIES` in `frontend/src/main.rs`, `areaServed` in `frontend/index.html`, `content/business_profile.toml`, [SERVICE_AREAS.md](SERVICE_AREAS.md) |
| "Where do you serve?" answer | `WHERE_WE_SERVE` in `backend/src/seo.rs` | FAQ text in `frontend/src/main.rs` and `frontend/index.html` |
| Business name, phone, email, social links | `BUSINESS` in `backend/src/seo.rs` | `seed_content()` in `backend/src/main.rs`, `frontend/index.html`, footer/header in `frontend/src/main.rs`, `content/business_profile.toml` |
| A service page (copy, FAQ, image) | `SERVICES` and the `*_FAQ` consts in `backend/src/seo.rs` | `seed_content()` in `backend/src/main.rs`; Yew service pages in `frontend/src/main.rs` |
| A resource article | `ARTICLES` in `backend/src/seo.rs` | `frontend/public/sitemap.xml` (regenerate) |
| Page titles, descriptions, H1s | The page function in `backend/src/seo.rs` (`home()`, `service_areas_index()`, ...) | `set_page_title()` in `frontend/src/main.rs` |
| JSON-LD / structured data | `local_business_schema()`, `service_area_schema()`, `*_schema()` in `backend/src/seo.rs` | Static fallback graph in `frontend/index.html` |
| Which URLs exist / sitemap | `indexable_paths()` + `page_for_path()` in `backend/src/seo.rs` | `frontend/public/sitemap.xml` |
| Redirects and `410 Gone` URLs | `obsolete_redirect()`, `is_gone_path()` in `backend/src/seo.rs` | [SEO_REDIRECT_MAP.md](SEO_REDIRECT_MAP.md) |
| Sitemap `lastmod` | `SITE_LASTMOD` (pages) or an article's `modified` in `backend/src/seo.rs` | `frontend/public/sitemap.xml` (test fails until they match) |
| Page layout / shared HTML (header, footer, contact form) | `render_page()`, `header()`, `footer()`, `contact_form()` in `backend/src/seo.rs` | Matching Yew components in `frontend/src/main.rs` |
| Styles | `frontend/styles.css` (served un-hashed, revalidated hourly) | — |
| Images | `frontend/public/assets/` (ship AVIF + WebP pairs) | `PageImage` entries in `backend/src/seo.rs` |
| Inquiry form API, validation, storage | `create_inquiry*`, `validate_inquiry()`, `Store` in `backend/src/main.rs` | — |
| Admin login (Microsoft Entra) | `microsoft_*` functions in `backend/src/main.rs` | README "Admin Login" section |
| Admin UI | `AdminPage` in `frontend/src/main.rs` | — |
| Analytics / Search Console hooks | `verification_and_analytics_hooks()` in `backend/src/seo.rs` | [SEARCH_ENGINE_SETUP.md](SEARCH_ENGINE_SETUP.md) |
| Environment variables | `backend/.env.example` | README |
| CI / deploy | `.github/workflows/` (`frontend.yml`, `seo.yml`, `deploy.yml`, `scorecard.yml`) | — |
| Route-level SEO audit | `scripts/seo-audit.sh` (CI) / `scripts/seo-audit.ps1` (Windows) | — |

## Source files

| Path | What it is |
|---|---|
| `backend/src/seo.rs` | Public-site source of truth: business profile, service areas, services, articles, every marketing page's HTML, JSON-LD, sitemap, robots, redirects, static-file serving, and most tests. |
| `backend/src/main.rs` | Rocket setup (`build_rocket()` mounts every route), `/api/*` inquiry + admin endpoints, Microsoft OAuth, SurrealDB/memory `Store`, `/api/site` seed content. |
| `backend/src/{config,db,models,routes}.rs` | Legacy template scaffolding. **Not compiled** — `main.rs` declares only `mod seo;`. Don't edit these expecting an effect. |
| `frontend/src/main.rs` | Yew SPA: client-side mirrors of the public pages, and the admin UI. |
| `frontend/src/api.rs` | Small fetch helpers for the Yew app. |
| `frontend/index.html` | Trunk entry point and the static fallback `<head>`/JSON-LD for deployments that serve `dist` directly. |
| `frontend/public/` | Copied into `dist` by Trunk: images, `robots.txt`, `sitemap.xml` fallbacks. |
| `content/business_profile.toml` | Human-readable record of what the owner has confirmed. Not read by code. |
| `docs/` | This map, SEO architecture and audits, checklists, content guardrails. |

## Rules that tests enforce

`cargo test -p backend` fails if any of these break, so check them before
editing copy in `seo.rs`:

- Titles ≤ 60 characters; descriptions 120–158 characters; all unique.
- Exactly one `<h1>` per page, no skipped heading levels.
- Service and article `answer` passages are 35–70 words and end with a period.
- A given FAQ question appears in `FAQPage` markup on one URL only.
- `CITY_LIST` names every `SERVICE_AREAS` entry, in order.
- Vermilion's `containedInPlace` lists both Lorain and Erie counties.
- Every service area has an anchor on `/service-areas`, appears in `areaServed`,
  and `/service-areas/{slug}` 301s to `/service-areas`.
- `frontend/public/robots.txt` and `sitemap.xml` match the generated output.
- JSON-LD has no street address, coordinates or `PostalAddress` (the business
  publishes no address).

## Validation

```sh
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
trunk build --release --config frontend/Trunk.toml   # needs trunk + wasm32 target
scripts/seo-audit.sh http://127.0.0.1:9001            # against a running server
```

## Related docs

- [SERVICE_AREAS.md](SERVICE_AREAS.md) — the Lorain County advertising geography and how to change it.
- [SEO_ARCHITECTURE.md](SEO_ARCHITECTURE.md) — how the SEO layer is built.
- [CONTENT_REQUIREMENTS.md](CONTENT_REQUIREMENTS.md) — claims that need owner confirmation.
- [SEO_REDIRECT_MAP.md](SEO_REDIRECT_MAP.md) — retired URLs.
- [SEO_RELEASE_CHECKLIST.md](SEO_RELEASE_CHECKLIST.md), [LOCAL_SEO_CHECKLIST.md](LOCAL_SEO_CHECKLIST.md), [SEARCH_ENGINE_SETUP.md](SEARCH_ENGINE_SETUP.md).
- [SEO_AUDIT.md](SEO_AUDIT.md) — historical audit log.
