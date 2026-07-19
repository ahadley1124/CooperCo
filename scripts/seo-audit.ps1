param(
    [string]$BaseUrl = "http://127.0.0.1:9001"
)

$ErrorActionPreference = "Stop"

$routes = @(
    "/",
    "/about",
    "/services",
    "/services/dog-training",
    "/services/puppy-training",
    "/services/group-dog-classes",
    "/contact",
    "/faq",
    "/service-areas",
    "/service-areas/lorain-oh",
    "/resources",
    "/privacy",
    "/accessibility"
)

foreach ($route in $routes) {
    $response = Invoke-WebRequest -Uri "$BaseUrl$route" -Method GET
    if ($response.StatusCode -ne 200) {
        throw "$route returned $($response.StatusCode)"
    }
    foreach ($required in @("<title>", "canonical", "<h1", "application/ld+json", "/contact")) {
        if ($response.Content -notlike "*$required*") {
            throw "$route is missing $required"
        }
    }
}

foreach ($route in @("/robots.txt", "/sitemap.xml")) {
    $response = Invoke-WebRequest -Uri "$BaseUrl$route" -Method GET
    if ($response.StatusCode -ne 200) {
        throw "$route returned $($response.StatusCode)"
    }
}

"SEO audit passed for $BaseUrl"
