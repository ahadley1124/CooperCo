#!/usr/bin/env bash
# Route-level SEO audit against a running Rocket server.
#
# A POSIX counterpart to seo-audit.ps1, so the same checks run on Linux CI
# runners and not only on a Windows workstation.
#
#   scripts/seo-audit.sh [base-url]        # default http://127.0.0.1:9001

set -euo pipefail

BASE_URL="${1:-http://127.0.0.1:9001}"
FAILURES=0

fail() {
	printf '  FAIL  %s\n' "$1" >&2
	FAILURES=$((FAILURES + 1))
}

pass() {
	printf '  ok    %s\n' "$1"
}

# Never route localhost checks through a proxy.
curl_raw() {
	curl -s --noproxy '*' "$@"
}

# The network helpers below absorb curl's exit status on purpose. Under
# `set -e` a connection error inside a command substitution kills the whole
# script, which would stop the audit at the first unreachable URL instead of
# reporting it and carrying on. curl still prints an HTTP code of 000, and the
# explicit checks below turn that into a counted failure.

status_of() {
	curl_raw -o /dev/null -w '%{http_code}' "$BASE_URL$1" || true
}

header_of() {
	{ curl_raw -o /dev/null -D - "$BASE_URL$1" || true; } |
		awk -v name="$(printf '%s' "$2" | tr '[:upper:]' '[:lower:]')" \
			'tolower($0) ~ "^" name ":" { sub(/^[^:]*: */, ""); gsub(/\r/, ""); print; exit }'
}

body_of() {
	curl_raw "$BASE_URL$1" || true
}

echo "SEO audit: $BASE_URL"

echo "- marketing routes"
ROUTES=$(body_of /sitemap.xml | grep -o '<loc>[^<]*</loc>' | sed 's|<loc>[^/]*//[^/]*||; s|</loc>||' || true)
if [ -z "$ROUTES" ]; then
	fail "sitemap.xml listed no URLs"
fi
for route in $ROUTES; do
	[ -z "$route" ] && route=/
	code=$(status_of "$route")
	if [ "$code" != "200" ]; then
		fail "$route returned $code"
		continue
	fi
	page=$(body_of "$route")
	missing=
	for required in '<title>' 'rel="canonical"' '<h1' 'application/ld+json' '/contact'; do
		case "$page" in
		*"$required"*) ;;
		*) missing="$missing $required" ;;
		esac
	done
	if [ -n "$missing" ]; then
		fail "$route is missing:$missing"
	else
		pass "$route"
	fi
done

echo "- robots and sitemap"
for route in /robots.txt /sitemap.xml; do
	code=$(status_of "$route")
	[ "$code" = "200" ] && pass "$route" || fail "$route returned $code"
done

sitemap=$(body_of /sitemap.xml)
for blocked in beta.cooper-and-co.com /service-areas/lorain-oh mansfield ontario lexington bellville ashland galion; do
	case "$(printf '%s' "$sitemap" | tr '[:upper:]' '[:lower:]')" in
	*"$blocked"*) fail "sitemap contains blocked value $blocked" ;;
	esac
done

robots=$(body_of /robots.txt)
for required in 'Disallow: /admin' 'Disallow: /api/' 'Disallow: /auth/' 'Sitemap: https://cooper-and-co.com/sitemap.xml'; do
	case "$robots" in
	*"$required"*) ;;
	*) fail "robots.txt is missing '$required'" ;;
	esac
done

echo "- redirects and retired URLs"
location=$(header_of /service-area/lorain-oh Location)
code=$(status_of /service-area/lorain-oh)
if [ "$code" = "301" ] && [ "$location" = "/service-areas" ]; then
	pass "/service-area/lorain-oh -> /service-areas"
else
	fail "/service-area/lorain-oh returned $code -> '$location', expected 301 -> /service-areas"
fi

for duplicate in /contact/ /faq/ /services/dog-training/; do
	code=$(status_of "$duplicate")
	location=$(header_of "$duplicate" Location)
	expected=${duplicate%/}
	if [ "$code" = "301" ] && [ "$location" = "$expected" ]; then
		pass "$duplicate -> $expected"
	else
		fail "$duplicate returned $code -> '$location', expected 301 -> $expected"
	fi
done

code=$(status_of /index.html)
location=$(header_of /index.html Location)
if [ "$code" = "301" ] && [ "$location" = "/" ]; then
	pass "/index.html -> /"
else
	fail "/index.html returned $code -> '$location', expected 301 -> /"
fi

code=$(status_of /service-area/mansfield-oh)
[ "$code" = "410" ] && pass "retired service-area URL is 410" ||
	fail "obsolete service-area URL returned $code, expected 410"

echo "- unknown routes"
code=$(status_of /not-a-real-cooperco-page)
if [ "$code" != "404" ]; then
	fail "unknown route returned $code, expected 404"
else
	case "$(body_of /not-a-real-cooperco-page)" in
	*'dog training and pet services in Lorain County'*)
		fail "unknown route returned homepage content"
		;;
	*) pass "unknown route is a real 404" ;;
	esac
fi

echo "- caching"
cache=$(header_of /styles.css Cache-Control)
case "$cache" in
*immutable*) fail "/styles.css is unversioned but cached immutably ($cache)" ;;
"") fail "/styles.css sent no Cache-Control" ;;
*) pass "/styles.css: $cache" ;;
esac

if [ "$FAILURES" -eq 0 ]; then
	echo "SEO audit passed for $BASE_URL"
else
	echo "SEO audit failed for $BASE_URL: $FAILURES problem(s)" >&2
	exit 1
fi
