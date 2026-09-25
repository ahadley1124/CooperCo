# AGENTS.md

Guidance for AI coding agents (and humans) working in this repository.

## Start here

1. **Read [`docs/PROJECT_MAP.md`](docs/PROJECT_MAP.md) first.** It maps every
   kind of change to the file and function that owns it, and lists the rules
   the tests enforce. Use it instead of searching the whole repo.
2. For anything touching location, cities or local SEO, read
   [`docs/SERVICE_AREAS.md`](docs/SERVICE_AREAS.md).
3. For claims about the business (services, prices, hours, credentials),
   check [`docs/CONTENT_REQUIREMENTS.md`](docs/CONTENT_REQUIREMENTS.md) and
   `content/business_profile.toml`.

## The business

Cooper & Co. is a dog-training and pet-service company based in **Lorain
County, Ohio** (phone (440) 276-1716, no public street address). Published
services: dog training, puppy training, group dog classes.

**Advertise primarily to Lorain County cities**: Elyria, Lorain, North
Ridgeville, Avon Lake, Avon, Amherst, Oberlin, Sheffield Lake, and Vermilion,
then the county's villages (Wellington, Grafton, LaGrange, Sheffield, South
Amherst, Kipton, Rochester). Don't target Cleveland/Cuyahoga County or the
retired Mansfield-area towns.

## Architecture in one paragraph

Rust workspace. `backend/` is a Rocket server that renders every public
marketing page as full HTML from `backend/src/seo.rs` (the source of truth for
business data, service areas, services, articles, metadata, JSON-LD, sitemap,
robots and redirects), plus `/api/*` inquiry/admin endpoints and Microsoft
Entra admin login in `backend/src/main.rs`. `frontend/` is a Yew app built by
Trunk that serves as progressive enhancement and the `/admin` UI; its public
copy mirrors `seo.rs` by hand. `backend/src/{config,db,models,routes}.rs` are
unused template leftovers (not declared as modules).

## Keeping copies in sync

Some facts are duplicated because the Yew app and the static fallbacks can't
import from the backend. When you change one, change all:

| Fact | Locations |
|---|---|
| City list | `SERVICE_AREAS` + `lorain_county_cities!()` (seo.rs), `LORAIN_COUNTY_CITIES` (frontend/src/main.rs), `areaServed` (frontend/index.html), `content/business_profile.toml` |
| Contact details | `BUSINESS` (seo.rs), `seed_content()` (main.rs), frontend/index.html, frontend/src/main.rs |
| Sitemap | generated in seo.rs; `frontend/public/sitemap.xml` must match (a test checks) |
| Robots | generated in seo.rs; `frontend/public/robots.txt` and `robots` must match |

## Rules

- Don't publish unverified claims: new services, prices, hours, testimonials,
  credentials, insurance, addresses, or per-city availability details.
- Don't add thin per-city landing pages; cities are anchored cards on
  `/service-areas` (see docs/SERVICE_AREAS.md for when a real city page is OK).
- Keep meta titles ≤ 60 chars and descriptions 120–158 chars, unique per page.
- Service/article opening `answer` passages: 35–70 words.
- Never add `PostalAddress`, street address or coordinates to JSON-LD.
- When page content changes, bump `SITE_LASTMOD` in seo.rs and regenerate
  `frontend/public/sitemap.xml` to match.
- Secrets live in `backend/.env` (see `backend/.env.example`); never commit them.

## Commands

```sh
cargo fmt --all -- --check
cargo test --workspace                 # backend tests carry the SEO rules
cargo clippy --workspace --all-targets --all-features -- -D warnings
trunk build --release --config frontend/Trunk.toml   # needs trunk + wasm32-unknown-unknown
cargo run -p backend                   # serves http://127.0.0.1:9001
scripts/seo-audit.sh http://127.0.0.1:9001
```

CI: `frontend.yml` (PRs: frontend tests + Trunk build), `seo.yml` (route audit
against a built server), `deploy.yml` (main: self-hosted build), `scorecard.yml`.

## Keep the docs current

If you change where something lives, add a route, or change the advertised
geography, update `docs/PROJECT_MAP.md`, `docs/SERVICE_AREAS.md` and this file
in the same commit.
