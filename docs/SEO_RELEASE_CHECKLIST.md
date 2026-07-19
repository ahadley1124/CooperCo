# SEO Release Checklist

- Run `cargo fmt --all -- --check`.
- Run `cargo check --workspace`.
- Run `cargo test --workspace`.
- Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Run `trunk build --release --config frontend/Trunk.toml`.
- Start Rocket from the production build.
- Inspect raw HTML for every sitemap URL.
- Verify `/robots.txt` and `/sitemap.xml`.
- Verify `/admin`, `/api/*`, and `/auth/*` are not indexable.
- Verify obsolete URLs redirect or return `410 Gone`.
- Check a nonexistent page returns `404`.
- Validate JSON-LD.
- Run Lighthouse, axe, HTML validation, and broken-link checking where available.
- Submit the sitemap after deployment.
