# SEO Redirect Map

| Old URL pattern | New URL | Status | Rationale |
|---|---:|---:|---|
| `/service-area` | `/service-areas` | 301 | Replace singular route with the current consolidated service-area page. |
| `/service-area/{current-lorain-county-city}` | `/service-areas` | 301 | Current city examples resolve to the consolidated service-area page instead of thin city pages. |
| `/service-areas/{current-lorain-county-city}` | `/service-areas` | 301 | Retire prior city-page URLs in favor of the consolidated service-area page. |
| Prior non-Lorain service-area URLs | none | 410 | Retire obsolete geography without implying an equivalent current destination. |
| `/services/dog-walking` | none | 410 | Service was not verified for publication. |
| `/services/pet-sitting` | none | 410 | Service was not verified for publication. |
| `/services/house-sitting` | none | 410 | Service was not verified for publication. |
| `/services/puppy-care` | none | 410 | Replaced by the current puppy-training page. |
| `/services/dog-adventures` | none | 410 | Service was not verified for publication. |
| Prior retired thin resource URLs | none | 410 | Retired resource pages without equivalent current content. |

Unknown URLs that are not listed here return `404 Not Found`.
