use std::{
    collections::HashMap,
    env,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Context;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use rand::{rngs::OsRng, RngCore};
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
    time::Duration,
    Build, Request, Rocket, State,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
const ADMIN_OAUTH_COOKIE: &str = "cooperco_ms_oauth";
const MICROSOFT_AUTHORITY: &str = "https://login.microsoftonline.com";
const MICROSOFT_SCOPES: &str = "openid profile email User.Read";

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

#[derive(Debug, Clone)]
struct MicrosoftOAuthConfig {
    client_id: String,
    client_secret: Option<String>,
    tenant: String,
    redirect_uri: String,
    post_login_redirect_uri: String,
    behind_proxy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MicrosoftOAuthContext {
    state: String,
    nonce: String,
    code_verifier: String,
    redirect_uri: String,
    created_at: u64,
}

#[derive(Debug, Serialize)]
struct AdminProfile {
    email: String,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MicrosoftTokenResponse {
    access_token: String,
    id_token: String,
}

#[derive(Debug, Deserialize)]
struct MicrosoftTokenError {
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MicrosoftUser {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    mail: Option<String>,
    #[serde(rename = "userPrincipalName")]
    user_principal_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MicrosoftIdTokenClaims {
    aud: String,
    exp: u64,
    iss: String,
    nonce: String,
    tid: Option<String>,
    email: Option<String>,
    preferred_username: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MicrosoftJwks {
    keys: Vec<MicrosoftJwk>,
}

#[derive(Debug, Deserialize)]
struct MicrosoftJwk {
    kid: Option<String>,
    kty: String,
    n: String,
    e: String,
    alg: Option<String>,
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
        .enumerate()
        .flat_map(|(site_index, site_url)| {
            paths.iter().map(move |path| {
                let priority = if site_index == 0 {
                    if *path == "/" { "1.0" } else { "0.8" }
                } else if *path == "/" {
                    "0.6"
                } else {
                    "0.5"
                };

                format!(
                    "<url><loc>{site_url}{path}</loc><changefreq>monthly</changefreq><priority>{priority}</priority></url>"
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
    let config = microsoft_oauth_config()?;
    let state = random_urlsafe(32);
    let nonce = random_urlsafe(32);
    let code_verifier = random_urlsafe(32);
    let code_challenge = pkce_challenge(&code_verifier);
    let context = MicrosoftOAuthContext {
        state: state.clone(),
        nonce: nonce.clone(),
        code_verifier,
        redirect_uri: config.redirect_uri.clone(),
        created_at: unix_timestamp(),
    };
    let context_json = serde_json::to_string(&context).map_err(|error| {
        status::Custom(
            Status::InternalServerError,
            format!("oauth context error: {error}"),
        )
    })?;

    cookies.add_private(oauth_cookie(
        ADMIN_OAUTH_COOKIE,
        context_json,
        Duration::minutes(10),
    ));

    let url = microsoft_authorization_url(&config, &state, &nonce, &code_challenge);
    log_oauth_event(
        "auth_start",
        &[
            ("tenant", config.tenant.as_str()),
            ("redirect_uri", config.redirect_uri.as_str()),
            (
                "behind_proxy",
                if config.behind_proxy { "true" } else { "false" },
            ),
            ("state_id", short_fingerprint(&state).as_str()),
            ("nonce_id", short_fingerprint(&nonce).as_str()),
            (
                "pkce_challenge_id",
                short_fingerprint(&code_challenge).as_str(),
            ),
            ("target", url.as_str()),
        ],
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
        return Err(microsoft_callback_error(error, error_description));
    }

    log_oauth_event(
        "callback_received",
        &[
            ("has_code", if code.is_some() { "true" } else { "false" }),
            (
                "state_id",
                state.map(short_fingerprint).unwrap_or_default().as_str(),
            ),
        ],
    );

    let context_json = cookies
        .get_private(ADMIN_OAUTH_COOKIE)
        .map(|cookie| cookie.value().to_owned())
        .ok_or_else(|| {
            log_oauth_event("state_validation", &[("result", "missing_context")]);
            status::Custom(Status::Unauthorized, "missing login state".to_owned())
        })?;
    let context: MicrosoftOAuthContext = serde_json::from_str(&context_json).map_err(|error| {
        log_oauth_event("state_validation", &[("result", "invalid_context")]);
        status::Custom(
            Status::Unauthorized,
            format!("invalid login state: {error}"),
        )
    })?;

    if let Err(reason) = validate_oauth_context(&context, state, unix_timestamp()) {
        cookies.remove_private(remove_oauth_cookie(ADMIN_OAUTH_COOKIE));
        let expected_state_id = short_fingerprint(&context.state);
        let received_state_id = state.map(short_fingerprint).unwrap_or_default();
        log_oauth_event(
            "state_validation",
            &[
                ("result", reason),
                ("expected_state_id", expected_state_id.as_str()),
                ("received_state_id", received_state_id.as_str()),
            ],
        );
        return Err(status::Custom(
            Status::Unauthorized,
            format!("invalid login state: {reason}"),
        ));
    }

    cookies.remove_private(remove_oauth_cookie(ADMIN_OAUTH_COOKIE));
    log_oauth_event(
        "state_validation",
        &[
            ("result", "ok"),
            ("state_id", short_fingerprint(&context.state).as_str()),
        ],
    );

    let code = require_auth_code(code)?;
    let config = microsoft_oauth_config()?;
    if context.redirect_uri != config.redirect_uri {
        log_oauth_event(
            "redirect_uri_validation",
            &[
                ("result", "mismatch"),
                ("stored", context.redirect_uri.as_str()),
                ("configured", config.redirect_uri.as_str()),
            ],
        );
        return Err(status::Custom(
            Status::Unauthorized,
            "login redirect URI changed during sign-in".to_owned(),
        ));
    }

    let token = exchange_microsoft_code(code, &context.code_verifier, &config).await?;
    let claims = validate_microsoft_id_token(&token.id_token, &config, &context.nonce).await?;
    let user = fetch_microsoft_user(&token.access_token).await?;
    let email = user
        .mail
        .or(user.user_principal_name)
        .or(claims.email)
        .or(claims.preferred_username)
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
        name: user.display_name.or(claims.name),
    };
    let session_json = serde_json::to_string(&session).map_err(|error| {
        status::Custom(
            Status::InternalServerError,
            format!("session error: {error}"),
        )
    })?;

    cookies.add_private(session_cookie(ADMIN_SESSION_COOKIE, session_json));
    log_oauth_event("session_created", &[("email", session.email.as_str())]);
    log_oauth_event(
        "final_redirect",
        &[("target", config.post_login_redirect_uri.as_str())],
    );

    Ok(Redirect::to(config.post_login_redirect_uri))
}

#[post("/auth/logout")]
fn logout(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove_private(remove_oauth_cookie(ADMIN_SESSION_COOKIE));
    log_oauth_event("logout", &[("result", "session_cleared")]);
    Redirect::to(post_login_redirect_uri())
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

#[get("/api/<_..>", rank = 15)]
fn api_not_found() -> Status {
    Status::NotFound
}

#[get("/auth/<_..>", rank = 15)]
fn auth_not_found() -> Status {
    Status::NotFound
}

#[get("/<_..>", rank = 20)]
async fn spa_fallback() -> Option<NamedFile> {
    frontend_index().await
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    load_dotenv_files();
    let store = connect_store().await;

    build_rocket(store)
        .launch()
        .await
        .context("rocket failed")?;

    Ok(())
}

fn build_rocket(store: Store) -> Rocket<Build> {
    rocket::build()
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
                api_not_found,
                auth_not_found,
                spa_fallback
            ],
        )
        .mount("/", FileServer::from(static_dir()).rank(10))
}

async fn frontend_index() -> Option<NamedFile> {
    NamedFile::open(static_dir().join("index.html")).await.ok()
}

fn load_dotenv_files() {
    if !load_dotenv_paths(&dotenv_candidate_paths()) {
        eprintln!("No backend .env file found; using shell environment and defaults");
    }
}

fn load_dotenv_paths(paths: &[PathBuf]) -> bool {
    let shell_env = env::vars().collect::<HashMap<_, _>>();
    let mut loaded = false;

    for path in paths {
        if path.is_file() {
            match dotenvy::from_path_override(&path) {
                Ok(_) => {
                    eprintln!("Loaded backend environment from {}", path.display());
                    loaded = true;
                }
                Err(error) => {
                    eprintln!("Could not load {}: {error}", path.display());
                }
            }
        }
    }

    for (key, value) in shell_env {
        if env::var_os(&key).is_none() {
            env::set_var(key, value);
        }
    }

    loaded
}

fn dotenv_candidate_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(current_dir) = env::current_dir() {
        paths.push(current_dir.join(".env"));
        paths.push(current_dir.join("backend").join(".env"));
    }

    paths
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
    code_verifier: &str,
    config: &MicrosoftOAuthConfig,
) -> Result<MicrosoftTokenResponse, status::Custom<String>> {
    let token_url = format!(
        "{}/{}/oauth2/v2.0/token",
        MICROSOFT_AUTHORITY, config.tenant
    );
    let mut form = vec![
        ("client_id", config.client_id.clone()),
        ("code", code.to_owned()),
        ("redirect_uri", config.redirect_uri.clone()),
        ("grant_type", "authorization_code".to_owned()),
        ("code_verifier", code_verifier.to_owned()),
    ];
    if let Some(client_secret) = &config.client_secret {
        form.push(("client_secret", client_secret.clone()));
    }

    log_oauth_event(
        "token_exchange",
        &[
            ("result", "request"),
            ("tenant", config.tenant.as_str()),
            ("redirect_uri", config.redirect_uri.as_str()),
            (
                "uses_client_secret",
                if config.client_secret.is_some() {
                    "true"
                } else {
                    "false"
                },
            ),
        ],
    );

    let response = reqwest::Client::new()
        .post(token_url)
        .form(&form)
        .send()
        .await
        .map_err(auth_server_error)?;
    let status_code = response.status();
    let body = response.text().await.map_err(auth_server_error)?;
    parse_microsoft_token_response(status_code, &body)
}

fn parse_microsoft_token_response(
    status_code: reqwest::StatusCode,
    body: &str,
) -> Result<MicrosoftTokenResponse, status::Custom<String>> {
    if !status_code.is_success() {
        let microsoft_error = serde_json::from_str::<MicrosoftTokenError>(body).ok();
        log_oauth_event(
            "token_exchange",
            &[
                ("result", "error"),
                ("status", status_code.as_str()),
                (
                    "error",
                    microsoft_error
                        .as_ref()
                        .and_then(|error| error.error.as_deref())
                        .unwrap_or("unknown"),
                ),
                (
                    "description",
                    microsoft_error
                        .as_ref()
                        .and_then(|error| error.error_description.as_deref())
                        .unwrap_or(""),
                ),
            ],
        );
        return Err(status::Custom(
            Status::Unauthorized,
            "Microsoft token exchange failed".to_owned(),
        ));
    }

    let token = serde_json::from_str::<MicrosoftTokenResponse>(body).map_err(|error| {
        log_oauth_event("token_exchange", &[("result", "invalid_json")]);
        status::Custom(
            Status::InternalServerError,
            format!("Microsoft token response was invalid: {error}"),
        )
    })?;
    log_oauth_event("token_exchange", &[("result", "ok")]);
    Ok(token)
}

async fn fetch_microsoft_user(access_token: &str) -> Result<MicrosoftUser, status::Custom<String>> {
    log_oauth_event("user_fetch", &[("result", "request")]);
    let user = reqwest::Client::new()
        .get("https://graph.microsoft.com/v1.0/me")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(auth_server_error)?
        .error_for_status()
        .map_err(auth_server_error)?
        .json::<MicrosoftUser>()
        .await
        .map_err(auth_server_error)?;
    log_oauth_event("user_fetch", &[("result", "ok")]);
    Ok(user)
}

async fn validate_microsoft_id_token(
    id_token: &str,
    config: &MicrosoftOAuthConfig,
    expected_nonce: &str,
) -> Result<MicrosoftIdTokenClaims, status::Custom<String>> {
    let header = decode_header(id_token).map_err(|error| {
        log_oauth_event("id_token_validation", &[("result", "invalid_header")]);
        status::Custom(
            Status::Unauthorized,
            format!("Microsoft ID token header was invalid: {error}"),
        )
    })?;
    let kid = header.kid.ok_or_else(|| {
        log_oauth_event("id_token_validation", &[("result", "missing_kid")]);
        status::Custom(
            Status::Unauthorized,
            "Microsoft ID token was missing a key id".to_owned(),
        )
    })?;
    if header.alg != Algorithm::RS256 {
        log_oauth_event("id_token_validation", &[("result", "invalid_alg")]);
        return Err(status::Custom(
            Status::Unauthorized,
            "Microsoft ID token used an unexpected algorithm".to_owned(),
        ));
    }

    let jwks_url = format!(
        "{}/{}/discovery/v2.0/keys",
        MICROSOFT_AUTHORITY, config.tenant
    );
    let jwks = reqwest::Client::new()
        .get(jwks_url)
        .send()
        .await
        .map_err(auth_server_error)?
        .error_for_status()
        .map_err(auth_server_error)?
        .json::<MicrosoftJwks>()
        .await
        .map_err(auth_server_error)?;
    let jwk = jwks
        .keys
        .into_iter()
        .find(|key| key.kid.as_deref() == Some(kid.as_str()) && key.kty == "RSA")
        .ok_or_else(|| {
            log_oauth_event("id_token_validation", &[("result", "missing_jwk")]);
            status::Custom(
                Status::Unauthorized,
                "Microsoft signing key was not found".to_owned(),
            )
        })?;
    if jwk.alg.as_deref().is_some_and(|alg| alg != "RS256") {
        log_oauth_event("id_token_validation", &[("result", "invalid_jwk_alg")]);
        return Err(status::Custom(
            Status::Unauthorized,
            "Microsoft signing key used an unexpected algorithm".to_owned(),
        ));
    }

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[config.client_id.as_str()]);
    let claims = decode::<MicrosoftIdTokenClaims>(
        id_token,
        &DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|error| {
            log_oauth_event("id_token_validation", &[("result", "invalid_jwk")]);
            status::Custom(
                Status::Unauthorized,
                format!("Microsoft signing key was invalid: {error}"),
            )
        })?,
        &validation,
    )
    .map_err(|error| {
        log_oauth_event(
            "id_token_validation",
            &[("result", "signature_or_claims_failed")],
        );
        status::Custom(
            Status::Unauthorized,
            format!("Microsoft ID token validation failed: {error}"),
        )
    })?
    .claims;

    validate_id_token_claims(&claims, config, expected_nonce)?;
    log_oauth_event(
        "id_token_validation",
        &[
            ("result", "ok"),
            ("issuer", claims.iss.as_str()),
            ("tenant_id", claims.tid.as_deref().unwrap_or("")),
        ],
    );
    Ok(claims)
}

fn validate_id_token_claims(
    claims: &MicrosoftIdTokenClaims,
    config: &MicrosoftOAuthConfig,
    expected_nonce: &str,
) -> Result<(), status::Custom<String>> {
    if claims.aud != config.client_id {
        log_oauth_event("id_token_validation", &[("result", "invalid_audience")]);
        return Err(status::Custom(
            Status::Unauthorized,
            "Microsoft ID token audience was invalid".to_owned(),
        ));
    }
    if claims.exp <= unix_timestamp() {
        log_oauth_event("id_token_validation", &[("result", "expired")]);
        return Err(status::Custom(
            Status::Unauthorized,
            "Microsoft ID token was expired".to_owned(),
        ));
    }
    if claims.nonce != expected_nonce {
        log_oauth_event("id_token_validation", &[("result", "invalid_nonce")]);
        return Err(status::Custom(
            Status::Unauthorized,
            "Microsoft ID token nonce was invalid".to_owned(),
        ));
    }
    if !issuer_allowed(&claims.iss, claims.tid.as_deref(), &config.tenant) {
        log_oauth_event("id_token_validation", &[("result", "invalid_issuer")]);
        return Err(status::Custom(
            Status::Unauthorized,
            "Microsoft ID token issuer was invalid".to_owned(),
        ));
    }

    Ok(())
}

fn validate_oauth_context(
    context: &MicrosoftOAuthContext,
    received_state: Option<&str>,
    now: u64,
) -> Result<(), &'static str> {
    if context.created_at + 600 < now {
        return Err("expired");
    }

    if received_state != Some(context.state.as_str()) {
        return Err("mismatch");
    }

    Ok(())
}

fn require_auth_code(code: Option<&str>) -> Result<&str, status::Custom<String>> {
    code.ok_or_else(|| status::Custom(Status::BadRequest, "missing auth code".to_owned()))
}

fn microsoft_callback_error(
    error: &str,
    error_description: Option<&str>,
) -> status::Custom<String> {
    log_oauth_event(
        "callback_microsoft_error",
        &[
            ("error", error),
            ("description", error_description.unwrap_or("")),
        ],
    );
    status::Custom(
        Status::Unauthorized,
        error_description.unwrap_or(error).to_owned(),
    )
}

fn issuer_allowed(issuer: &str, token_tenant: Option<&str>, configured_tenant: &str) -> bool {
    let issuer = issuer.trim_end_matches('/');
    if let Some(token_tenant) = token_tenant {
        let expected = format!("{MICROSOFT_AUTHORITY}/{token_tenant}/v2.0");
        if issuer == expected {
            return true;
        }
    }

    !matches!(configured_tenant, "common" | "organizations" | "consumers")
        && issuer == format!("{MICROSOFT_AUTHORITY}/{configured_tenant}/v2.0")
}

fn microsoft_oauth_config() -> Result<MicrosoftOAuthConfig, status::Custom<String>> {
    let client_id = required_env("MICROSOFT_CLIENT_ID")?;
    let tenant = env::var("MICROSOFT_TENANT_ID").unwrap_or_else(|_| "common".to_owned());
    let redirect_uri = microsoft_redirect_uri();
    let post_login_redirect_uri = post_login_redirect_uri();

    Ok(MicrosoftOAuthConfig {
        client_id,
        client_secret: env::var("MICROSOFT_CLIENT_SECRET")
            .ok()
            .filter(|secret| !secret.trim().is_empty()),
        tenant,
        redirect_uri,
        post_login_redirect_uri,
        behind_proxy: bool_env("BEHIND_PROXY"),
    })
}

fn required_env(name: &str) -> Result<String, status::Custom<String>> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            status::Custom(
                Status::InternalServerError,
                format!("{name} is required for Microsoft admin login"),
            )
        })
}

fn microsoft_redirect_uri() -> String {
    env::var("MICROSOFT_REDIRECT_URI")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("{}/auth/microsoft/callback", backend_base_url()))
}

fn post_login_redirect_uri() -> String {
    env::var("MICROSOFT_POST_LOGIN_REDIRECT_URI")
        .or_else(|_| env::var("POST_LOGIN_REDIRECT_URI"))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("{}/admin", public_app_url()))
}

fn backend_base_url() -> String {
    env::var("BACKEND_BASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:9001".to_owned())
        .trim_end_matches('/')
        .to_owned()
}

fn public_app_url() -> String {
    env::var("PUBLIC_APP_URL")
        .or_else(|_| env::var("PUBLIC_SITE_URL"))
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "http://127.0.0.1:9001".to_owned())
        .trim_end_matches('/')
        .to_owned()
}

fn microsoft_authorization_url(
    config: &MicrosoftOAuthConfig,
    state: &str,
    nonce: &str,
    code_challenge: &str,
) -> String {
    format!(
        "{}/{}/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&response_mode=query&scope={}&state={}&nonce={}&code_challenge={}&code_challenge_method=S256",
        MICROSOFT_AUTHORITY,
        config.tenant,
        percent_encode(&config.client_id),
        percent_encode(&config.redirect_uri),
        percent_encode(MICROSOFT_SCOPES),
        percent_encode(state),
        percent_encode(nonce),
        percent_encode(code_challenge),
    )
}

fn random_urlsafe(byte_len: usize) -> String {
    let mut bytes = vec![0_u8; byte_len];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn pkce_challenge(code_verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()))
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn oauth_cookie(name: &'static str, value: String, max_age: Duration) -> Cookie<'static> {
    let mut builder = Cookie::build((name, value))
        .http_only(true)
        .secure(cookie_secure())
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(max_age);
    if let Some(domain) = cookie_domain() {
        builder = builder.domain(domain);
    }
    builder.build()
}

fn session_cookie(name: &'static str, value: String) -> Cookie<'static> {
    oauth_cookie(name, value, Duration::days(7))
}

fn remove_oauth_cookie(name: &'static str) -> Cookie<'static> {
    let mut builder = Cookie::build(name)
        .path("/")
        .secure(cookie_secure())
        .same_site(SameSite::Lax);
    if let Some(domain) = cookie_domain() {
        builder = builder.domain(domain);
    }
    builder.build()
}

fn cookie_secure() -> bool {
    env::var("COOKIE_SECURE")
        .map(|value| truthy(&value))
        .unwrap_or_else(|_| public_app_url().starts_with("https://"))
}

fn bool_env(name: &str) -> bool {
    env::var(name).map(|value| truthy(&value)).unwrap_or(false)
}

fn truthy(value: &str) -> bool {
    matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "YES")
}

fn cookie_domain() -> Option<String> {
    env::var("COOKIE_DOMAIN")
        .ok()
        .map(|domain| domain.trim().to_owned())
        .filter(|domain| !domain.is_empty())
}

fn short_fingerprint(value: &str) -> String {
    URL_SAFE_NO_PAD
        .encode(Sha256::digest(value.as_bytes()))
        .chars()
        .take(12)
        .collect()
}

fn log_oauth_event(event: &str, fields: &[(&str, &str)]) {
    let details = fields
        .iter()
        .map(|(key, value)| format!("{key}={}", value.replace(['\n', '\r'], " ")))
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!("oauth event={event} {details}");
}

fn public_site_urls() -> Vec<String> {
    env::var("PUBLIC_SITE_URLS")
        .or_else(|_| env::var("PUBLIC_SITE_URL"))
        .unwrap_or_else(|_| "https://cooper-and-co.com,https://beta.cooper-and-co.com".to_owned())
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

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::local::blocking::Client;
    use std::fs;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn oauth_config() -> MicrosoftOAuthConfig {
        MicrosoftOAuthConfig {
            client_id: "client-id".to_owned(),
            client_secret: None,
            tenant: "organizations".to_owned(),
            redirect_uri: "http://127.0.0.1:9001/auth/microsoft/callback".to_owned(),
            post_login_redirect_uri: "http://127.0.0.1:9001/admin".to_owned(),
            behind_proxy: false,
        }
    }

    fn auth_test_client() -> Client {
        Client::tracked(
            rocket::build().mount("/", routes![microsoft_login, microsoft_callback, logout]),
        )
        .expect("valid rocket")
    }

    fn full_app_test_client(static_dir: &std::path::Path) -> Client {
        env::set_var("COOPERCO_STATIC_DIR", static_dir);
        Client::tracked(build_rocket(Store::Memory(Arc::new(RwLock::new(
            Vec::new(),
        )))))
        .expect("valid rocket")
    }

    #[test]
    fn rocket_serves_frontend_index_and_assets_without_trunk() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let dir = env::temp_dir().join(format!("cooperco-static-test-{}", Uuid::new_v4()));
        fs::create_dir_all(dir.join("assets")).unwrap();
        fs::write(
            dir.join("index.html"),
            "<!doctype html><html><body>Cooper frontend</body></html>",
        )
        .unwrap();
        fs::write(dir.join("assets").join("app.css"), "body { color: black; }").unwrap();

        let client = full_app_test_client(&dir);
        let index = client.get("/").dispatch();
        assert_eq!(index.status(), Status::Ok);
        assert!(index.into_string().unwrap().contains("Cooper frontend"));

        let fallback = client.get("/admin").dispatch();
        assert_eq!(fallback.status(), Status::Ok);
        assert!(fallback.into_string().unwrap().contains("Cooper frontend"));

        let asset = client.get("/assets/app.css").dispatch();
        assert_eq!(asset.status(), Status::Ok);
        assert_eq!(
            asset.headers().get_one("Content-Type"),
            Some("text/css; charset=utf-8")
        );
        assert!(asset.into_string().unwrap().contains("color: black"));

        env::remove_var("COOPERCO_STATIC_DIR");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rocket_does_not_spa_fallback_unknown_api_or_auth_routes() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let dir = env::temp_dir().join(format!("cooperco-route-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("index.html"), "<html>frontend</html>").unwrap();

        let client = full_app_test_client(&dir);

        assert_eq!(
            client.get("/api/missing").dispatch().status(),
            Status::NotFound
        );
        assert_eq!(
            client.get("/auth/missing").dispatch().status(),
            Status::NotFound
        );
        assert_eq!(
            client.get("/not-a-backend-route").dispatch().status(),
            Status::Ok
        );

        env::remove_var("COOPERCO_STATIC_DIR");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn dotenv_file_values_override_shell_and_shell_fills_missing_keys() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let dir = env::temp_dir().join(format!("cooperco-dotenv-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env");
        fs::write(
            &path,
            "COOPERCO_TEST_DOTENV_OVERRIDE=file-value\nCOOPERCO_TEST_DOTENV_FILE_ONLY=file-only\n",
        )
        .unwrap();

        env::set_var("COOPERCO_TEST_DOTENV_OVERRIDE", "shell-value");
        env::set_var("COOPERCO_TEST_DOTENV_SHELL_ONLY", "shell-only");
        env::remove_var("COOPERCO_TEST_DOTENV_FILE_ONLY");

        assert!(load_dotenv_paths(&[path]));
        assert_eq!(
            env::var("COOPERCO_TEST_DOTENV_OVERRIDE").as_deref(),
            Ok("file-value")
        );
        assert_eq!(
            env::var("COOPERCO_TEST_DOTENV_FILE_ONLY").as_deref(),
            Ok("file-only")
        );
        assert_eq!(
            env::var("COOPERCO_TEST_DOTENV_SHELL_ONLY").as_deref(),
            Ok("shell-only")
        );

        env::remove_var("COOPERCO_TEST_DOTENV_OVERRIDE");
        env::remove_var("COOPERCO_TEST_DOTENV_FILE_ONLY");
        env::remove_var("COOPERCO_TEST_DOTENV_SHELL_ONLY");
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn pkce_challenge_matches_rfc7636_vector() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

        assert_eq!(
            pkce_challenge(verifier),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn authorization_url_includes_pkce_nonce_state_and_backend_callback() {
        let config = oauth_config();
        let url = microsoft_authorization_url(&config, "state value", "nonce value", "challenge");

        assert!(url
            .starts_with("https://login.microsoftonline.com/organizations/oauth2/v2.0/authorize?"));
        assert!(url.contains("client_id=client-id"));
        assert!(url.contains("response_type=code"));
        assert!(url
            .contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A9001%2Fauth%2Fmicrosoft%2Fcallback"));
        assert!(url.contains("scope=openid%20profile%20email%20User.Read"));
        assert!(url.contains("state=state%20value"));
        assert!(url.contains("nonce=nonce%20value"));
        assert!(url.contains("code_challenge=challenge"));
        assert!(url.contains("code_challenge_method=S256"));
    }

    #[test]
    fn login_route_redirects_to_microsoft_and_sets_oauth_cookie() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        env::set_var("MICROSOFT_CLIENT_ID", "client-id");
        env::set_var("MICROSOFT_TENANT_ID", "organizations");
        env::set_var("BACKEND_BASE_URL", "http://127.0.0.1:9001");
        env::set_var("PUBLIC_APP_URL", "http://127.0.0.1:9001");
        env::remove_var("MICROSOFT_REDIRECT_URI");

        let client = auth_test_client();
        let response = client.get("/auth/microsoft/login").dispatch();

        assert_eq!(response.status(), Status::SeeOther);
        let location = response.headers().get_one("Location").unwrap();
        assert!(location
            .starts_with("https://login.microsoftonline.com/organizations/oauth2/v2.0/authorize?"));
        assert!(location
            .contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A9001%2Fauth%2Fmicrosoft%2Fcallback"));
        assert!(location.contains("code_challenge_method=S256"));
        let set_cookie = response.headers().get("Set-Cookie").collect::<Vec<_>>();
        assert!(set_cookie
            .iter()
            .any(|cookie| cookie.contains(ADMIN_OAUTH_COOKIE)));

        env::remove_var("MICROSOFT_CLIENT_ID");
        env::remove_var("MICROSOFT_TENANT_ID");
        env::remove_var("BACKEND_BASE_URL");
        env::remove_var("PUBLIC_APP_URL");
    }

    #[test]
    fn default_redirect_uri_uses_backend_base_url_not_old_8080_port() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        env::remove_var("MICROSOFT_REDIRECT_URI");
        env::remove_var("BACKEND_BASE_URL");

        assert_eq!(
            microsoft_redirect_uri(),
            "http://127.0.0.1:9001/auth/microsoft/callback"
        );
    }

    #[test]
    fn configured_redirect_uri_takes_precedence() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        env::set_var(
            "MICROSOFT_REDIRECT_URI",
            "https://api.example.com/auth/microsoft/callback",
        );
        env::set_var("BACKEND_BASE_URL", "http://127.0.0.1:9001");

        assert_eq!(
            microsoft_redirect_uri(),
            "https://api.example.com/auth/microsoft/callback"
        );

        env::remove_var("MICROSOFT_REDIRECT_URI");
        env::remove_var("BACKEND_BASE_URL");
    }

    #[test]
    fn oauth_config_records_when_backend_is_behind_proxy() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        env::set_var("MICROSOFT_CLIENT_ID", "client-id");
        env::set_var("BEHIND_PROXY", "true");

        let config = microsoft_oauth_config().unwrap();

        assert!(config.behind_proxy);

        env::remove_var("MICROSOFT_CLIENT_ID");
        env::remove_var("BEHIND_PROXY");
    }

    #[test]
    fn id_token_claim_validation_rejects_bad_nonce() {
        let config = oauth_config();
        let claims = MicrosoftIdTokenClaims {
            aud: config.client_id.clone(),
            exp: unix_timestamp() + 60,
            iss: "https://login.microsoftonline.com/tenant-id/v2.0".to_owned(),
            nonce: "actual".to_owned(),
            tid: Some("tenant-id".to_owned()),
            email: None,
            preferred_username: None,
            name: None,
        };

        assert!(validate_id_token_claims(&claims, &config, "expected").is_err());
    }

    #[test]
    fn id_token_claim_validation_accepts_tenant_issuer_audience_expiry_and_nonce() {
        let config = oauth_config();
        let claims = MicrosoftIdTokenClaims {
            aud: config.client_id.clone(),
            exp: unix_timestamp() + 60,
            iss: "https://login.microsoftonline.com/tenant-id/v2.0".to_owned(),
            nonce: "nonce".to_owned(),
            tid: Some("tenant-id".to_owned()),
            email: Some("admin@example.com".to_owned()),
            preferred_username: None,
            name: Some("Admin".to_owned()),
        };

        assert!(validate_id_token_claims(&claims, &config, "nonce").is_ok());
    }

    #[test]
    fn oauth_context_validation_accepts_matching_state() {
        let context = MicrosoftOAuthContext {
            state: "state".to_owned(),
            nonce: "nonce".to_owned(),
            code_verifier: "verifier".to_owned(),
            redirect_uri: "http://127.0.0.1:9001/auth/microsoft/callback".to_owned(),
            created_at: 100,
        };

        assert_eq!(validate_oauth_context(&context, Some("state"), 120), Ok(()));
    }

    #[test]
    fn oauth_context_validation_rejects_state_mismatch_and_expiry() {
        let context = MicrosoftOAuthContext {
            state: "state".to_owned(),
            nonce: "nonce".to_owned(),
            code_verifier: "verifier".to_owned(),
            redirect_uri: "http://127.0.0.1:9001/auth/microsoft/callback".to_owned(),
            created_at: 100,
        };

        assert_eq!(
            validate_oauth_context(&context, Some("other"), 120),
            Err("mismatch")
        );
        assert_eq!(
            validate_oauth_context(&context, Some("state"), 701),
            Err("expired")
        );
    }

    #[test]
    fn missing_auth_code_returns_bad_request() {
        let error = require_auth_code(None).unwrap_err();

        assert_eq!(error.0, Status::BadRequest);
        assert_eq!(error.1, "missing auth code");
    }

    #[test]
    fn microsoft_error_callback_returns_visible_unauthorized_message() {
        let error = microsoft_callback_error("access_denied", Some("User denied access"));

        assert_eq!(error.0, Status::Unauthorized);
        assert_eq!(error.1, "User denied access");
    }

    #[test]
    fn token_exchange_error_returns_unauthorized_without_exposing_response_body() {
        let error = parse_microsoft_token_response(
            reqwest::StatusCode::BAD_REQUEST,
            r#"{"error":"invalid_grant","error_description":"AADSTS bad code"}"#,
        )
        .unwrap_err();

        assert_eq!(error.0, Status::Unauthorized);
        assert_eq!(error.1, "Microsoft token exchange failed");
    }

    #[test]
    fn session_cookie_uses_http_only_lax_path_max_age_and_local_secure_default() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        env::remove_var("COOKIE_SECURE");
        env::remove_var("COOKIE_DOMAIN");
        env::remove_var("PUBLIC_APP_URL");
        env::remove_var("PUBLIC_SITE_URL");

        let cookie = session_cookie(ADMIN_SESSION_COOKIE, "session".to_owned());

        assert_eq!(cookie.name(), ADMIN_SESSION_COOKIE);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(false));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        assert!(cookie.max_age().is_some());
    }

    #[test]
    fn logout_removal_cookie_matches_session_cookie_scope() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        env::remove_var("COOKIE_SECURE");
        env::remove_var("COOKIE_DOMAIN");
        env::remove_var("PUBLIC_APP_URL");
        env::remove_var("PUBLIC_SITE_URL");

        let cookie = remove_oauth_cookie(ADMIN_SESSION_COOKIE);

        assert_eq!(cookie.name(), ADMIN_SESSION_COOKIE);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.secure(), Some(false));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
    }

    #[test]
    fn logout_route_redirects_to_configured_admin_url() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        env::remove_var("COOKIE_SECURE");
        env::remove_var("COOKIE_DOMAIN");
        env::set_var("PUBLIC_APP_URL", "http://127.0.0.1:9001");

        let client = auth_test_client();
        let response = client.post("/auth/logout").dispatch();

        assert_eq!(response.status(), Status::SeeOther);
        assert_eq!(
            response.headers().get_one("Location"),
            Some("http://127.0.0.1:9001/admin")
        );
        env::remove_var("PUBLIC_APP_URL");
    }
}
