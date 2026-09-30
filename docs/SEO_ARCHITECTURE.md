# SEO Architecture

`backend/src/seo.rs` is the SEO source of truth for public marketing routes.

It defines:

- Business profile fields.
- Confirmed services.
- Lorain County service areas (`SERVICE_AREAS`, `CITY_LIST`) and the consolidated service-area page.
- Resource articles.
- Route-specific metadata.
- JSON-LD.
- Sitemap generation.
- Robots generation.
- Obsolete URL handling.

Rocket mounts public marketing routes before the catchall noindex admin shell. Static assets are served by the same route layer when the path contains a file extension.

Every page's JSON-LD graph is self-contained: `render_page()` emits the organization, website and image nodes on all pages so that `@id` references resolve within the document, then de-duplicates by `@id`.

`frontend/public/robots.txt` and `frontend/public/sitemap.xml` are fallbacks for deployments that serve `dist` directly. A test pins them to the generated output; regenerate them from a running server rather than editing them by hand.

Indexable routes must exist in `indexable_paths()` and have a matching `page_for_path()` renderer. Individual city pages are not generated. `/service-areas` gives every Lorain County city an anchored card (`/service-areas#elyria-oh`); the homepage and service pages link to those anchors. See `docs/SERVICE_AREAS.md`.

The Yew frontend is now enhancement and admin UI. Search engines do not need WebAssembly to see essential public content.
