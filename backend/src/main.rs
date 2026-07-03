use std::{env, path::PathBuf, sync::Arc};

use anyhow::Context;
use rocket::{
    delete,
    fs::{FileServer, NamedFile},
    get,
    http::{Cookie, CookieJar, HeaderMap, SameSite, Status},
    patch, post,
    request::{FromRequest, Outcome},
    response::content::{RawText, RawXml},
    response::status,
    response::Redirect,
    routes,
    serde::json::Json,
    Config, Request, State,
};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    types::SurrealValue,
    Surreal,
};
use tokio::sync::RwLock;
use uuid::Uuid;

type Db = Surreal<Client>;
const ADMIN_SESSION_COOKIE: &str = "cooperco_admin_session";
const ADMIN_STATE_COOKIE: &str = "cooperco_ms_state";

#[derive(Clone)]
enum Store {
    Surreal(Arc<Db>),
    Memory(Arc<RwLock<Vec<Inquiry>>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SiteContent {
    business: Business,
    stats: Vec<Stat>,
    services: Vec<Service>,
    updates: Vec<Update>,
    gallery: Vec<GalleryImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Business {
    name: String,
    category: String,
    location: String,
    phone: String,
    email: String,
    facebook_url: String,
    yelp_url: String,
    intro: String,
    hero_image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Stat {
    label: String,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Service {
    title: String,
    summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Update {
    title: String,
    summary: String,
    source_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GalleryImage {
    src: String,
    alt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct Inquiry {
    id: Uuid,
    name: String,
    email: String,
    phone: String,
    pet_name: String,
    message: String,
    #[serde(default = "default_inquiry_status")]
    status: String,
}

#[derive(Debug, Clone, Deserialize)]
struct NewInquiry {
    name: String,
    email: String,
    phone: String,
    pet_name: String,
    message: String,
}

#[derive(Debug, Clone, Deserialize)]
struct InquiryStatusUpdate {
    status: String,
}

#[derive(Debug, Clone, SurrealValue)]
#[surreal(crate = "surrealdb::types")]
struct InquiryStatusPatch {
    status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AdminSession {
    email: String,
    name: Option<String>,
}

#[derive(Debug, Serialize)]
struct AdminProfile {
    email: String,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MicrosoftTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct MicrosoftUser {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    mail: Option<String>,
    #[serde(rename = "userPrincipalName")]
    user_principal_name: Option<String>,
}

#[derive(Debug)]
struct AdminAuth {
    email: String,
    name: Option<String>,
}

#[derive(Debug)]
enum AuthError {
    Missing,
}

#[derive(Debug, Serialize)]
struct Health {
    ok: bool,
    store: &'static str,
}

#[get("/api/health")]
fn health(store: &State<Store>) -> Json<Health> {
    let store_name = match store.inner() {
        Store::Surreal(_) => "surrealdb",
        Store::Memory(_) => "memory",
    };

    Json(Health {
        ok: true,
        store: store_name,
    })
}

#[get("/api/site")]
fn site_content() -> Json<SiteContent> {
    Json(seed_content())
}

#[get("/robots.txt")]
fn robots_txt() -> RawText<String> {
    let body = if noindex_enabled() {
        "User-agent: *\nDisallow: /\n".to_owned()
    } else {
        let sitemaps = public_site_urls()
            .into_iter()
            .map(|site_url| format!("Sitemap: {site_url}/sitemap.xml\n"))
            .collect::<String>();
        format!("User-agent: *\nAllow: /\n{sitemaps}")
    };

    RawText(body)
}

#[get("/robots")]
fn robots() -> RawText<String> {
    robots_txt()
}

#[get("/sitemap.xml")]
fn sitemap_xml() -> RawXml<String> {
    let site_urls = public_site_urls();
    let paths = [
        "/",
        "/services",
        "/group-classes",
        "/contact",
        "/service-area/lorain-county-oh",
        "/service-area/elyria-oh",
        "/service-area/lorain-oh",
        "/service-area/amherst-oh",
        "/service-area/avon-oh",
        "/service-area/north-ridgeville-oh",
    ];

    let urls = site_urls
        .iter()
        .flat_map(|site_url| {
            paths.iter().map(move |path| {
                format!(
                    "<url><loc>{site_url}{path}</loc><changefreq>monthly</changefreq><priority>{priority}</priority></url>",
                    priority = if *path == "/" { "1.0" } else { "0.8" }
                )
            })
        })
        .collect::<String>();

    RawXml(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">{urls}</urlset>"#
    ))
}

#[get("/auth/microsoft/login")]
fn microsoft_login(cookies: &CookieJar<'_>) -> Result<Redirect, status::Custom<String>> {
    let client_id = env::var("MICROSOFT_CLIENT_ID").map_err(|_| {
        status::Custom(
            Status::InternalServerError,
            "MICROSOFT_CLIENT_ID is required for admin login".to_owned(),
        )
    })?;

    let tenant = microsoft_tenant();
    let redirect_uri = microsoft_redirect_uri();
    let state = Uuid::new_v4().to_string();
    cookies.add_private(
        Cookie::build((ADMIN_STATE_COOKIE, state.clone()))
            .http_only(true)
            .same_site(SameSite::Lax)
            .path("/")
            .build(),
    );

    let url = format!(
        "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize?client_id={client_id}&response_type=code&redirect_uri={redirect_uri}&response_mode=query&scope=openid%20profile%20email%20User.Read&state={state}",
        redirect_uri = percent_encode(&redirect_uri),
        state = percent_encode(&state),
    );

    Ok(Redirect::to(url))
}

#[get("/auth/microsoft/callback?<code>&<state>&<error>&<error_description>")]
async fn microsoft_callback(
    cookies: &CookieJar<'_>,
    code: Option<&str>,
    state: Option<&str>,
    error: Option<&str>,
    error_description: Option<&str>,
) -> Result<Redirect, status::Custom<String>> {
    if let Some(error) = error {
        return Err(status::Custom(
            Status::Unauthorized,
            error_description.unwrap_or(error).to_owned(),
        ));
    }

    let expected_state = cookies
        .get_private(ADMIN_STATE_COOKIE)
        .map(|cookie| cookie.value().to_owned())
        .ok_or_else(|| status::Custom(Status::Unauthorized, "missing login state".to_owned()))?;

    if state != Some(expected_state.as_str()) {
        return Err(status::Custom(
            Status::Unauthorized,
            "invalid login state".to_owned(),
        ));
    }

    cookies.remove_private(Cookie::build(ADMIN_STATE_COOKIE).path("/").build());

    let code =
        code.ok_or_else(|| status::Custom(Status::BadRequest, "missing auth code".to_owned()))?;
    let token = exchange_microsoft_code(code).await?;
    let user = fetch_microsoft_user(&token.access_token).await?;
    let email = user
        .mail
        .or(user.user_principal_name)
        .ok_or_else(|| {
            status::Custom(
                Status::Forbidden,
                "Microsoft account has no email".to_owned(),
            )
        })?
        .to_ascii_lowercase();

    if !admin_email_allowed(&email) {
        return Err(status::Custom(
            Status::Forbidden,
            "Microsoft account is not allowed to access admin".to_owned(),
        ));
    }

    let session = AdminSession {
        email,
        name: user.display_name,
    };
    let session_json = serde_json::to_string(&session).map_err(|error| {
        status::Custom(
            Status::InternalServerError,
            format!("session error: {error}"),
        )
    })?;

    cookies.add_private(
        Cookie::build((ADMIN_SESSION_COOKIE, session_json))
            .http_only(true)
            .same_site(SameSite::Lax)
            .path("/")
            .build(),
    );

    Ok(Redirect::to("/admin"))
}

#[post("/auth/logout")]
fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private(Cookie::build(ADMIN_SESSION_COOKIE).path("/").build());
    Redirect::to("/admin")
}

#[get("/api/admin/me")]
fn admin_me(admin: AdminAuth) -> Json<AdminProfile> {
    Json(AdminProfile {
        email: admin.email,
        name: admin.name,
    })
}

#[get("/api/admin/inquiries")]
async fn admin_inquiries(
    admin: AdminAuth,
    store: &State<Store>,
) -> Result<Json<Vec<Inquiry>>, status::Custom<String>> {
    let _ = admin;

    match store.inner() {
        Store::Surreal(db) => db.select("inquiry").await.map(Json).map_err(server_error),
        Store::Memory(items) => Ok(Json(items.read().await.clone())),
    }
}

#[post("/api/inquiries", format = "json", data = "<payload>")]
async fn create_inquiry(
    store: &State<Store>,
    payload: Json<NewInquiry>,
) -> Result<status::Created<Json<Inquiry>>, status::Custom<String>> {
    let new = payload.into_inner();
    validate_inquiry(&new)?;

    let inquiry = Inquiry {
        id: Uuid::new_v4(),
        name: new.name.trim().to_owned(),
        email: new.email.trim().to_owned(),
        phone: new.phone.trim().to_owned(),
        pet_name: new.pet_name.trim().to_owned(),
        message: new.message.trim().to_owned(),
        status: default_inquiry_status(),
    };

    match store.inner() {
        Store::Surreal(db) => {
            let created: Option<Inquiry> = db
                .create(("inquiry", inquiry.id.to_string()))
                .content(inquiry.clone())
                .await
                .map_err(server_error)?;

            let saved = created.unwrap_or(inquiry);
            Ok(status::Created::new("/api/inquiries").body(Json(saved)))
        }
        Store::Memory(items) => {
            items.write().await.push(inquiry.clone());
            Ok(status::Created::new("/api/inquiries").body(Json(inquiry)))
        }
    }
}

#[patch(
    "/api/admin/inquiries/<id>/status",
    format = "json",
    data = "<payload>"
)]
async fn update_inquiry_status(
    admin: AdminAuth,
    store: &State<Store>,
    id: &str,
    payload: Json<InquiryStatusUpdate>,
) -> Result<Json<Inquiry>, status::Custom<String>> {
    let _ = admin;
    let id = parse_inquiry_id(id)?;
    let next_status = normalize_inquiry_status(&payload.status)?;

    match store.inner() {
        Store::Surreal(db) => {
            let updated: Option<Inquiry> = db
                .update(("inquiry", id.to_string()))
                .merge(InquiryStatusPatch {
                    status: next_status,
                })
                .await
                .map_err(server_error)?;

            updated
                .map(Json)
                .ok_or_else(|| status::Custom(Status::NotFound, "inquiry was not found".to_owned()))
        }
        Store::Memory(items) => {
            let mut items = items.write().await;
            let Some(inquiry) = items.iter_mut().find(|item| item.id == id) else {
                return Err(status::Custom(
                    Status::NotFound,
                    "inquiry was not found".to_owned(),
                ));
            };

            inquiry.status = next_status;
            Ok(Json(inquiry.clone()))
        }
    }
}

#[delete("/api/admin/inquiries/<id>")]
async fn delete_inquiry(
    admin: AdminAuth,
    store: &State<Store>,
    id: &str,
) -> Result<Status, status::Custom<String>> {
    let _ = admin;
    let id = parse_inquiry_id(id)?;

    match store.inner() {
        Store::Surreal(db) => {
            let deleted: Option<Inquiry> = db
                .delete(("inquiry", id.to_string()))
                .await
                .map_err(server_error)?;

            if deleted.is_some() {
                Ok(Status::NoContent)
            } else {
                Err(status::Custom(
                    Status::NotFound,
                    "inquiry was not found".to_owned(),
                ))
            }
        }
        Store::Memory(items) => {
            let mut items = items.write().await;
            let original_len = items.len();
            items.retain(|item| item.id != id);

            if items.len() == original_len {
                Err(status::Custom(
                    Status::NotFound,
                    "inquiry was not found".to_owned(),
                ))
            } else {
                Ok(Status::NoContent)
            }
        }
    }
}

#[get("/<_..>", rank = 20)]
async fn spa_fallback() -> Option<NamedFile> {
    NamedFile::open(static_dir().join("index.html")).await.ok()
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let store = connect_store().await;

    rocket::build()
        .configure(Config {
            port: 9001,
            ..Config::debug_default()
        })
        .manage(store)
        .mount(
            "/",
            routes![
                health,
                site_content,
                robots_txt,
                robots,
                sitemap_xml,
                microsoft_login,
                microsoft_callback,
                logout,
                admin_me,
                admin_inquiries,
                update_inquiry_status,
                delete_inquiry,
                create_inquiry,
                spa_fallback
            ],
        )
        .mount("/", FileServer::from(static_dir()).rank(10))
        .launch()
        .await
        .context("rocket failed")?;

    Ok(())
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminAuth {
    type Error = AuthError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(admin) = admin_from_private_cookie(request.cookies()) {
            return Outcome::Success(admin);
        }

        if bearer_token_allowed(request.headers()) {
            return Outcome::Success(AdminAuth {
                email: "api-token".to_owned(),
                name: Some("Admin API token".to_owned()),
            });
        }

        Outcome::Error((Status::Unauthorized, AuthError::Missing))
    }
}

fn admin_from_private_cookie(cookies: &CookieJar<'_>) -> Option<AdminAuth> {
    let cookie = cookies.get_private(ADMIN_SESSION_COOKIE)?;
    let session: AdminSession = serde_json::from_str(cookie.value()).ok()?;

    if !admin_email_allowed(&session.email) {
        return None;
    }

    Some(AdminAuth {
        email: session.email,
        name: session.name,
    })
}

fn bearer_token_allowed(headers: &HeaderMap<'_>) -> bool {
    let Some(expected) = env::var("ADMIN_API_TOKEN")
        .ok()
        .filter(|token| !token.is_empty())
    else {
        return false;
    };

    headers
        .get_one("Authorization")
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(|token| token == expected)
        .unwrap_or(false)
}

async fn exchange_microsoft_code(
    code: &str,
) -> Result<MicrosoftTokenResponse, status::Custom<String>> {
    let client_id = env::var("MICROSOFT_CLIENT_ID").map_err(|_| {
        status::Custom(
            Status::InternalServerError,
            "MICROSOFT_CLIENT_ID is required".to_owned(),
        )
    })?;
    let client_secret = env::var("MICROSOFT_CLIENT_SECRET").map_err(|_| {
        status::Custom(
            Status::InternalServerError,
            "MICROSOFT_CLIENT_SECRET is required".to_owned(),
        )
    })?;
    let tenant = microsoft_tenant();
    let redirect_uri = microsoft_redirect_uri();
    let token_url = format!("https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token");

    reqwest::Client::new()
        .post(token_url)
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code),
            ("redirect_uri", redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(auth_server_error)?
        .error_for_status()
        .map_err(auth_server_error)?
        .json::<MicrosoftTokenResponse>()
        .await
        .map_err(auth_server_error)
}

async fn fetch_microsoft_user(access_token: &str) -> Result<MicrosoftUser, status::Custom<String>> {
    reqwest::Client::new()
        .get("https://graph.microsoft.com/v1.0/me")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(auth_server_error)?
        .error_for_status()
        .map_err(auth_server_error)?
        .json::<MicrosoftUser>()
        .await
        .map_err(auth_server_error)
}

fn microsoft_tenant() -> String {
    env::var("MICROSOFT_TENANT_ID").unwrap_or_else(|_| "common".to_owned())
}

fn microsoft_redirect_uri() -> String {
    env::var("MICROSOFT_REDIRECT_URI")
        .unwrap_or_else(|_| "http://127.0.0.1:8080/auth/microsoft/callback".to_owned())
}

fn public_site_urls() -> Vec<String> {
    env::var("PUBLIC_SITE_URLS")
        .or_else(|_| env::var("PUBLIC_SITE_URL"))
        .unwrap_or_else(|_| "https://beta.cooper-and-co.com,https://cooper-and-co.com".to_owned())
        .split(',')
        .map(|url| url.trim().trim_end_matches('/').to_owned())
        .filter(|url| !url.is_empty())
        .collect()
}

fn noindex_enabled() -> bool {
    env::var("COOPERCO_NOINDEX")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn admin_email_allowed(email: &str) -> bool {
    env::var("ADMIN_ALLOWED_EMAILS")
        .ok()
        .map(|allowed| {
            allowed
                .split(',')
                .map(|item| item.trim().to_ascii_lowercase())
                .any(|allowed_email| allowed_email == email)
        })
        .unwrap_or(false)
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

async fn connect_store() -> Store {
    let url = env::var("SURREALDB_URL").ok();
    let namespace = env::var("SURREALDB_NS").unwrap_or_else(|_| "cooperco".to_owned());
    let database = env::var("SURREALDB_DB").unwrap_or_else(|_| "website".to_owned());
    let username = env::var("SURREALDB_USER").ok();
    let password = env::var("SURREALDB_PASS").ok();

    let Some(url) = url else {
        return Store::Memory(Arc::new(RwLock::new(Vec::new())));
    };

    match Surreal::new::<Ws>(&url).await {
        Ok(db) => {
            if let (Some(username), Some(password)) = (username, password) {
                if let Err(error) = db.signin(Root { username, password }).await {
                    eprintln!("SurrealDB sign-in failed, using memory store: {error}");
                    return Store::Memory(Arc::new(RwLock::new(Vec::new())));
                }
            }

            if let Err(error) = db.use_ns(namespace).use_db(database).await {
                eprintln!(
                    "SurrealDB namespace/database selection failed, using memory store: {error}"
                );
                return Store::Memory(Arc::new(RwLock::new(Vec::new())));
            }

            Store::Surreal(Arc::new(db))
        }
        Err(error) => {
            eprintln!("SurrealDB connection failed, using memory store: {error}");
            Store::Memory(Arc::new(RwLock::new(Vec::new())))
        }
    }
}

fn validate_inquiry(inquiry: &NewInquiry) -> Result<(), status::Custom<String>> {
    let required = [
        ("name", inquiry.name.trim()),
        ("email", inquiry.email.trim()),
        ("message", inquiry.message.trim()),
    ];

    if let Some((field, _)) = required.iter().find(|(_, value)| value.is_empty()) {
        return Err(status::Custom(
            Status::BadRequest,
            format!("{field} is required"),
        ));
    }

    if !inquiry.email.contains('@') {
        return Err(status::Custom(
            Status::BadRequest,
            "email must include @".to_owned(),
        ));
    }

    Ok(())
}

fn default_inquiry_status() -> String {
    "submitted".to_owned()
}

fn parse_inquiry_id(id: &str) -> Result<Uuid, status::Custom<String>> {
    Uuid::parse_str(id)
        .map_err(|_| status::Custom(Status::BadRequest, "invalid inquiry id".to_owned()))
}

fn normalize_inquiry_status(status: &str) -> Result<String, status::Custom<String>> {
    let normalized = status.trim().to_ascii_lowercase();

    match normalized.as_str() {
        "submitted" | "contacted" | "purchased" => Ok(normalized),
        _ => Err(status::Custom(
            Status::BadRequest,
            "status must be submitted, contacted, or purchased".to_owned(),
        )),
    }
}

fn server_error(error: surrealdb::Error) -> status::Custom<String> {
    status::Custom(Status::InternalServerError, error.to_string())
}

fn auth_server_error(error: reqwest::Error) -> status::Custom<String> {
    let status = if error.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
        Status::Unauthorized
    } else {
        Status::InternalServerError
    };

    status::Custom(status, error.to_string())
}

fn static_dir() -> PathBuf {
    if let Ok(path) = env::var("COOPERCO_STATIC_DIR") {
        return PathBuf::from(path);
    }

    let from_workspace = PathBuf::from("frontend/dist");
    if from_workspace.is_dir() {
        return from_workspace;
    }

    PathBuf::from("../frontend/dist")
}

fn seed_content() -> SiteContent {
    SiteContent {
        business: Business {
            name: "Cooper & Co.".to_owned(),
            category: "Pet service".to_owned(),
            location: "Lorain County, OH".to_owned(),
            phone: "(440) 276-1716".to_owned(),
            email: "cooper.copetservices@gmail.com".to_owned(),
            facebook_url: "https://www.facebook.com/CooperAndCoPet".to_owned(),
            yelp_url: "https://m.yelp.com/biz/cooper-and-company-elyria".to_owned(),
            intro: "Cooper & Co. helps local pet families ask about dog training, group classes, puppy classes, and pet support across Lorain County, Elyria, Lorain, Amherst, Avon, and North Ridgeville, Ohio.".to_owned(),
            hero_image: "/assets/facebook-cooperco-hero.webp".to_owned(),
        },
        stats: vec![
            Stat {
                label: "Facebook likes".to_owned(),
                value: "177".to_owned(),
            },
            Stat {
                label: "Followers".to_owned(),
                value: "177".to_owned(),
            },
            Stat {
                label: "Reviews noted".to_owned(),
                value: "3".to_owned(),
            },
        ],
        services: vec![
            Service {
                title: "Group dog classes".to_owned(),
                summary: "Seasonal group classes help dogs practice calm focus, leash manners, and social learning around other pets.".to_owned(),
            },
            Service {
                title: "Puppy classes and training questions".to_owned(),
                summary: "Ask about age-appropriate puppy support, early manners, confidence building, and current class availability.".to_owned(),
            },
            Service {
                title: "Local pet service inquiries".to_owned(),
                summary: "Share your pet details, goals, schedule needs, and location so Cooper & Co. can respond directly.".to_owned(),
            },
        ],
        updates: vec![Update {
            title: "Ask about upcoming group dog classes".to_owned(),
            summary: "Class times and openings can change. Contact Cooper & Co. for the latest dog training and group class schedule.".to_owned(),
            source_label: "Current availability".to_owned(),
        }],
        gallery: vec![
            GalleryImage {
                src: "/assets/facebook-cooperco-gallery-1.webp".to_owned(),
                alt: "Cooper & Co. pet services logo from the public Facebook page".to_owned(),
            },
            GalleryImage {
                src: "/assets/facebook-cooperco-gallery-2.webp".to_owned(),
                alt: "Cooper & Co. pet services logo from the public Facebook page".to_owned(),
            },
            GalleryImage {
                src: "/assets/facebook-cooperco-gallery-3.webp".to_owned(),
                alt: "Cooper & Co. pet services logo from the public Facebook page".to_owned(),
            },
        ],
    }
}
