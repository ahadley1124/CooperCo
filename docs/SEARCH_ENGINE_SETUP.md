# Search Engine Setup

1. Verify `https://cooper-and-co.com` in Google Search Console.
2. Submit `https://cooper-and-co.com/sitemap.xml`.
3. Use URL Inspection for `/`, `/services/dog-training`, `/services/puppy-training`, `/services/group-dog-classes`, and `/service-areas`.
4. Check `site:cooper-and-co.com` indexing after release.
5. Confirm every indexed URL has the production canonical.
6. Validate JSON-LD with Google's Rich Results Test or Schema Markup Validator.
7. Review Core Web Vitals in Search Console after field data accumulates.
8. Verify Bing Webmaster Tools and submit the same sitemap.
9. Keep beta and staging deployments excluded with robots `Disallow: /` and `X-Robots-Tag: noindex, nofollow`.
10. Monitor crawl errors, redirects, sitemap coverage, and branded/local queries for 30 days after release.

Environment hooks:

- `GOOGLE_SITE_VERIFICATION` or `COOPERCO_SEARCH_CONSOLE_VERIFICATION`
- `BING_SITE_VERIFICATION`
- `GA4_MEASUREMENT_ID` or `GTM_CONTAINER_ID`
- `MICROSOFT_CLARITY_ID`
- `META_PIXEL_ID`

Analytics hooks are emitted only when configured IDs look valid. IndexNow is not implemented; do not claim that IndexNow guarantees indexing.
