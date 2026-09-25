# Service Areas: Lorain County

Cooper & Co. is a Lorain County, Ohio company. The site advertises **primarily
to the cities of Lorain County**, then to the county's villages. Copy, titles,
descriptions and structured data should name these places rather than
Cleveland-area or out-of-county locations.

## Advertised communities

Cities, in the order they are promoted (largest markets and the county seat
first):

| City | Slug | Anchor |
|---|---|---|
| Elyria (county seat) | `elyria-oh` | `/service-areas#elyria-oh` |
| Lorain (largest city) | `lorain-oh` | `/service-areas#lorain-oh` |
| North Ridgeville | `north-ridgeville-oh` | `/service-areas#north-ridgeville-oh` |
| Avon Lake | `avon-lake-oh` | `/service-areas#avon-lake-oh` |
| Avon | `avon-oh` | `/service-areas#avon-oh` |
| Amherst | `amherst-oh` | `/service-areas#amherst-oh` |
| Oberlin | `oberlin-oh` | `/service-areas#oberlin-oh` |
| Sheffield Lake | `sheffield-lake-oh` | `/service-areas#sheffield-lake-oh` |
| Vermilion (partly in Erie County) | `vermilion-oh` | `/service-areas#vermilion-oh` |

Villages: Wellington, Grafton, LaGrange, Sheffield, South Amherst, Kipton,
Rochester (slugs follow the same `name-oh` pattern).

## Where the geography appears

- **Homepage** — hero line lists every city; the "Serving every Lorain County
  city" section links each city to its card on `/service-areas`.
- **Service pages** — "{Service} across Lorain County" section with the same
  city links; the opening answer names every city.
- **`/service-areas`** — one card per city (with an `id` anchor and a factual
  location note) and a list of villages.
- **About, FAQ, `/api/site` intro** — name the cities via `CITY_LIST` /
  `WHERE_WE_SERVE`.
- **JSON-LD** — `LocalBusiness.areaServed` and each `Service.areaServed` list
  Lorain County plus every city and village as `City` nodes.
- **Meta descriptions** — each page names a different subset of cities, since
  descriptions must be unique and 120–158 characters.

## Why there are no per-city landing pages

The 2026-07 SEO remediation removed thin city pages (near-identical pages
differing only by the town name are treated as doorway pages). City URLs such
as `/service-areas/elyria-oh` now `301` to `/service-areas`, where each city has
its own anchored card. Add a dedicated city page only when it has genuinely
unique, owner-verified content for that city (a class location, local
testimonials, etc.), and add it to `indexable_paths()` and `page_for_path()`.

## Adding or removing a community

1. `backend/src/seo.rs`: edit `SERVICE_AREAS` (slug, name, `AreaKind`, one
   factual geography note). For a city, also edit `lorain_county_cities!()` —
   a test fails if the two disagree.
2. Adjust any meta description that names the city, keeping 120–158 characters.
3. `frontend/src/main.rs`: `LORAIN_COUNTY_CITIES`.
4. `frontend/index.html`: the static `areaServed` array.
5. `content/business_profile.toml`: the `[[service_areas]]` entry.
6. Update the tables above.
7. If a community is dropped, decide whether its old
   `/service-areas/{slug}` URL should keep redirecting (add it to
   `obsolete_redirect()`) or return `410` (`obsolete_non_lorain_slug()`).
8. `cargo test -p backend`.

Out-of-county slugs from the old site (Mansfield, Ontario, Lexington,
Bellville, Ashland, Galion) return `410 Gone` — do not reintroduce them.

## Guardrails

- Location notes describe geography only. Don't claim travel radius, pricing,
  class venues or response times per city unless the owner confirms them.
- No street address is published; the business is a service-area business.
