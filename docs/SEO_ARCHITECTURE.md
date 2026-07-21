# SEO Architecture

`backend/src/seo.rs` is the SEO source of truth for public marketing routes.

It defines:

- Business profile fields.
- Confirmed services.
- The consolidated Lorain County service-area page.
- Resource articles.
- Route-specific metadata.
- JSON-LD.
- Sitemap generation.
- Robots generation.
- Obsolete URL handling.

Rocket mounts public marketing routes before the catchall noindex admin shell. Static assets are served by the same route layer when the path contains a file extension.

Indexable routes must exist in `indexable_paths()` and have a matching `page_for_path()` renderer. Individual city pages are not generated; the service-area page lists Lorain County and the current city examples used on the visible site.

The Yew frontend is now enhancement and admin UI. Search engines do not need WebAssembly to see essential public content.
