param(
    [string]$BaseUrl = "http://127.0.0.1:9001"
)

$ErrorActionPreference = "Stop"

function Get-RouteResponse {
    param(
        [string]$Path,
        [int]$MaximumRedirection = 5
    )

    $request = [System.Net.HttpWebRequest]::Create("$BaseUrl$Path")
    $request.Method = "GET"
    $request.AllowAutoRedirect = $MaximumRedirection -gt 0

    try {
        $rawResponse = $request.GetResponse()
    } catch [System.Net.WebException] {
        $rawResponse = $_.Exception.Response
        if ($null -eq $rawResponse) {
            throw
        }
    }

    $reader = New-Object System.IO.StreamReader($rawResponse.GetResponseStream())
    return [PSCustomObject]@{
        StatusCode = [int]$rawResponse.StatusCode
        Headers = $rawResponse.Headers
        Content = $reader.ReadToEnd()
    }
}

function Get-RouteHeader {
    param(
        $Response,
        [string]$Name
    )

    return $Response.Headers[$Name]
}

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
    "/resources",
    "/privacy",
    "/accessibility"
)

foreach ($route in $routes) {
    $response = Get-RouteResponse -Path $route
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
    $response = Get-RouteResponse -Path $route
    if ($response.StatusCode -ne 200) {
        throw "$route returned $($response.StatusCode)"
    }
}

$sitemap = (Get-RouteResponse -Path "/sitemap.xml").Content
foreach ($blocked in @("beta.cooper-and-co.com", "/service-areas/lorain-oh", "mansfield", "ontario", "lexington", "bellville", "ashland", "galion")) {
    if ($sitemap -like "*$blocked*") {
        throw "sitemap contains blocked value $blocked"
    }
}

$robots = (Get-RouteResponse -Path "/robots.txt").Content
foreach ($required in @("Disallow: /admin", "Disallow: /api/", "Disallow: /auth/", "Sitemap: https://cooper-and-co.com/sitemap.xml")) {
    if ($robots -notlike "*$required*") {
        throw "robots.txt is missing $required"
    }
}

$redirect = Get-RouteResponse -Path "/service-area/lorain-oh" -MaximumRedirection 0
if ($redirect.StatusCode -ne 301 -or (Get-RouteHeader -Response $redirect -Name "Location") -ne "/service-areas") {
    throw "/service-area/lorain-oh did not 301 to /service-areas"
}

$retired = Get-RouteResponse -Path "/service-area/mansfield-oh"
if ($retired.StatusCode -ne 410) {
    throw "obsolete service-area URL returned $($retired.StatusCode), expected 410"
}

$unknown = Get-RouteResponse -Path "/not-a-real-cooperco-page"
if ($unknown.StatusCode -ne 404) {
    throw "unknown route returned $($unknown.StatusCode), expected 404"
}
if ($unknown.Content -like "*Cooper &amp; Co. dog training and pet services in Lorain County*") {
    throw "unknown route returned homepage content"
}

"SEO audit passed for $BaseUrl"
