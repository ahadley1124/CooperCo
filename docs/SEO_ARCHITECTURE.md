# SEO Architecture

`backend/src/seo.rs` is the SEO source of truth for public marketing routes.

It defines:

- Business profile fields.
- Confirmed services.
- Confirmed, unconfirmed, and not-served location status support.
- Resource articles.
- Route-specific metadata.
- JSON-LD.
- Sitemap generation.
- Robots generation.
- Obsolete URL handling.

Rocket mounts public marketing routes before the catchall noindex admin shell. Static assets are served by the same route layer when the path contains a file extension.

Indexable routes must exist in `indexable_paths()` and have a matching `page_for_path()` renderer. Location pages are only generated from service areas marked `Confirmed`.

The Yew frontend is now enhancement and admin UI. Search engines do not need WebAssembly to see essential public content.
