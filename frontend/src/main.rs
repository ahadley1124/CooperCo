use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct SiteContent {
    business: Business,
    stats: Vec<Stat>,
    services: Vec<Service>,
    updates: Vec<Update>,
    gallery: Vec<GalleryImage>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct Stat {
    label: String,
    value: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct Service {
    title: String,
    summary: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct Update {
    title: String,
    summary: String,
    source_label: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct GalleryImage {
    src: String,
    alt: String,
}

#[derive(Clone, Debug, Default, Serialize)]
struct InquiryForm {
    name: String,
    email: String,
    phone: String,
    pet_name: String,
    message: String,
}

#[derive(Clone, Debug, Default, Serialize)]
struct AdminLogin {
    username: String,
    password: String,
}

#[derive(Clone, Debug, Deserialize)]
struct AdminLoginResponse {
    token: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct Inquiry {
    id: String,
    name: String,
    email: String,
    phone: String,
    pet_name: String,
    message: String,
    status: String,
}

#[derive(Clone, Debug, Serialize)]
struct InquiryStatusUpdate {
    status: String,
}

#[function_component(App)]
fn app() -> Html {
    match current_route() {
        AppRoute::Admin => html! { <AdminPage /> },
        AppRoute::Public => html! { <PublicPage /> },
    }
}

enum AppRoute {
    Admin,
    Public,
}

fn current_route() -> AppRoute {
    let path = web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .unwrap_or_default();

    if path.trim_end_matches('/') == "/admin" {
        AppRoute::Admin
    } else {
        AppRoute::Public
    }
}

#[function_component(PublicPage)]
fn public_page() -> Html {
    let content = use_state(|| None::<SiteContent>);
    let load_error = use_state(|| None::<String>);
    let form = use_state(InquiryForm::default);
    let submit_state = use_state(|| "idle".to_owned());

    {
        let content = content.clone();
        let load_error = load_error.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match Request::get("/api/site").send().await {
                    Ok(response) if response.ok() => match response.json::<SiteContent>().await {
                        Ok(site) => content.set(Some(site)),
                        Err(error) => load_error.set(Some(error.to_string())),
                    },
                    Ok(response) => load_error.set(Some(format!(
                        "Content request failed with status {}",
                        response.status()
                    ))),
                    Err(error) => load_error.set(Some(error.to_string())),
                }
            });
            || ()
        });
    }

    let update_field = {
        let form = form.clone();
        move |field: &'static str| {
            let form = form.clone();
            Callback::from(move |event: InputEvent| {
                let value = event
                    .target_dyn_into::<HtmlInputElement>()
                    .map(|input| input.value())
                    .or_else(|| {
                        event
                            .target_dyn_into::<HtmlTextAreaElement>()
                            .map(|textarea| textarea.value())
                    })
                    .unwrap_or_default();

                let mut next = (*form).clone();
                match field {
                    "name" => next.name = value,
                    "email" => next.email = value,
                    "phone" => next.phone = value,
                    "pet_name" => next.pet_name = value,
                    "message" => next.message = value,
                    _ => {}
                }
                form.set(next);
            })
        }
    };

    let onsubmit = {
        let form = form.clone();
        let submit_state = submit_state.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let payload = (*form).clone();
            let form = form.clone();
            let submit_state = submit_state.clone();

            submit_state.set("sending".to_owned());
            spawn_local(async move {
                let builder =
                    Request::post("/api/inquiries").header("Content-Type", "application/json");
                let request = match builder.json(&payload) {
                    Ok(request) => request,
                    Err(error) => {
                        submit_state.set(format!("Could not prepare inquiry: {error}"));
                        return;
                    }
                };

                match request.send().await {
                    Ok(response) if response.ok() => {
                        form.set(InquiryForm::default());
                        submit_state.set("sent".to_owned());
                    }
                    Ok(response) => submit_state.set(format!(
                        "Please check the form. Status {}",
                        response.status()
                    )),
                    Err(error) => submit_state.set(format!("Could not send inquiry: {error}")),
                }
            });
        })
    };

    let Some(site) = (*content).clone() else {
        return html! {
            <div class="loading">
                <div class="mark">{"C&Co"}</div>
                <p>{load_error.as_ref().cloned().unwrap_or_else(|| "Loading Cooper & Co.".to_owned())}</p>
            </div>
        };
    };

    html! {
        <>
            <header class="topbar">
                <a class="brand" href="#top" aria-label="Cooper and Co home">
                    <span class="brand-mark">{"C&Co"}</span>
                    <span>{site.business.name.clone()}</span>
                </a>
                <nav aria-label="Main navigation">
                    <a href="#services">{"Services"}</a>
                    <a href="#updates">{"Updates"}</a>
                    <a href="#contact">{"Contact"}</a>
                </nav>
            </header>

            <section id="top" class="hero">
                <img class="hero-image" src={site.business.hero_image.clone()} alt="Cooper & Co. pets and services" />
                <div class="hero-copy">
                    <p class="eyebrow">{format!("{} in {}", site.business.category, site.business.location)}</p>
                    <h1>{site.business.name.clone()}</h1>
                    <p>{site.business.intro.clone()}</p>
                    <div class="hero-actions">
                        <a class="button primary" href="#contact">{"Request information"}</a>
                        <a class="button secondary" href={format!("tel:{}", site.business.phone.replace([' ', '(', ')', '-'], ""))}>{site.business.phone.clone()}</a>
                    </div>
                </div>
            </section>

            <section class="stats" aria-label="Facebook profile stats">
                {for site.stats.iter().map(|stat| html! {
                    <div class="stat">
                        <strong>{stat.value.clone()}</strong>
                        <span>{stat.label.clone()}</span>
                    </div>
                })}
            </section>

            <section id="services" class="section">
                <div class="section-heading">
                    <p class="eyebrow">{"What is available"}</p>
                    <h2>{"Pet services with direct local contact"}</h2>
                </div>
                <div class="service-grid">
                    {for site.services.iter().map(|service| html! {
                        <article class="card">
                            <h3>{service.title.clone()}</h3>
                            <p>{service.summary.clone()}</p>
                        </article>
                    })}
                </div>
            </section>

            <section id="updates" class="section split">
                <div>
                    <p class="eyebrow">{"Latest public update"}</p>
                    <h2>{"Class news from Cooper & Co."}</h2>
                </div>
                <div class="updates">
                    {for site.updates.iter().map(|update| html! {
                        <article class="update">
                            <span>{update.source_label.clone()}</span>
                            <h3>{update.title.clone()}</h3>
                            <p>{update.summary.clone()}</p>
                            <a href={site.business.facebook_url.clone()} target="_blank" rel="noreferrer">{"Open Facebook page"}</a>
                        </article>
                    })}
                </div>
            </section>

            <section class="gallery" aria-label="Cooper and Co photo preview">
                {for site.gallery.iter().map(|image| html! {
                    <img src={image.src.clone()} alt={image.alt.clone()} />
                })}
            </section>

            <section id="contact" class="section contact">
                <div class="contact-copy">
                    <p class="eyebrow">{"Contact"}</p>
                    <h2>{"Ask about classes or pet support"}</h2>
                    <a href={format!("mailto:{}", site.business.email)}>{site.business.email.clone()}</a>
                    <a href={format!("tel:{}", site.business.phone.replace([' ', '(', ')', '-'], ""))}>{site.business.phone.clone()}</a>
                    <a href={site.business.yelp_url.clone()} target="_blank" rel="noreferrer">{"Yelp listing"}</a>
                </div>
                <form onsubmit={onsubmit}>
                    <label>
                        {"Name"}
                        <input value={form.name.clone()} oninput={update_field("name")} required=true />
                    </label>
                    <label>
                        {"Email"}
                        <input r#type="email" value={form.email.clone()} oninput={update_field("email")} required=true />
                    </label>
                    <label>
                        {"Phone"}
                        <input value={form.phone.clone()} oninput={update_field("phone")} />
                    </label>
                    <label>
                        {"Pet name"}
                        <input value={form.pet_name.clone()} oninput={update_field("pet_name")} />
                    </label>
                    <label class="wide">
                        {"Message"}
                        <textarea value={form.message.clone()} oninput={update_field("message")} required=true />
                    </label>
                    <button class="button primary" type="submit" disabled={*submit_state == "sending"}>{"Send inquiry"}</button>
                    <p class="form-status">{match submit_state.as_str() {
                        "idle" => "",
                        "sending" => "Sending...",
                        "sent" => "Inquiry sent.",
                        other => other,
                    }}</p>
                </form>
            </section>

            <footer>
                <span>{format!("{} · {}", site.business.name, site.business.location)}</span>
                <a href={site.business.facebook_url} target="_blank" rel="noreferrer">{"Facebook"}</a>
            </footer>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(AdminPage)]
fn admin_page() -> Html {
    let login = use_state(AdminLogin::default);
    let inquiries = use_state(Vec::<Inquiry>::new);
    let status = use_state(|| None::<String>);
    let admin_token = use_state(|| None::<String>);

    let update_field = {
        let login = login.clone();
        move |field: &'static str| {
            let login = login.clone();
            Callback::from(move |event: InputEvent| {
                let value = event
                    .target_dyn_into::<HtmlInputElement>()
                    .map(|input| input.value())
                    .unwrap_or_default();

                let mut next = (*login).clone();
                match field {
                    "username" => next.username = value,
                    "password" => next.password = value,
                    _ => {}
                }
                login.set(next);
            })
        }
    };

    let onsubmit = {
        let login = login.clone();
        let inquiries = inquiries.clone();
        let status = status.clone();
        let admin_token = admin_token.clone();

        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let credentials = (*login).clone();
            let inquiries = inquiries.clone();
            let status = status.clone();
            let admin_token = admin_token.clone();

            status.set(Some("Signing in...".to_owned()));
            spawn_local(async move {
                match admin_login(credentials).await {
                    Ok(token) => match fetch_admin_inquiries(&token).await {
                        Ok(items) => {
                            inquiries.set(items);
                            admin_token.set(Some(token));
                            status.set(None);
                        }
                        Err(error) => status.set(Some(error)),
                    },
                    Err(error) => status.set(Some(error)),
                }
            });
        })
    };

    let onrefresh = {
        let admin_token = admin_token.clone();
        let inquiries = inquiries.clone();
        let status = status.clone();

        Callback::from(move |_| {
            let token = (*admin_token).clone();
            let inquiries = inquiries.clone();
            let status = status.clone();
            let admin_token = admin_token.clone();

            let Some(token) = token else {
                status.set(Some("Sign in again to refresh inquiries.".to_owned()));
                return;
            };

            status.set(Some("Refreshing inquiries...".to_owned()));
            spawn_local(async move {
                match fetch_admin_inquiries(&token).await {
                    Ok(items) => {
                        inquiries.set(items);
                        status.set(None);
                    }
                    Err(error) => {
                        if error.starts_with("Sign in again") {
                            admin_token.set(None);
                        }
                        status.set(Some(error));
                    }
                }
            });
        })
    };

    let update_status = {
        let admin_token = admin_token.clone();
        let inquiries = inquiries.clone();
        let status = status.clone();

        move |id: String, next_status: &'static str| {
            let token = (*admin_token).clone();
            let inquiries = inquiries.clone();
            let status = status.clone();
            let next_status = next_status.to_owned();

            Callback::from(move |_| {
                let Some(token) = token.clone() else {
                    status.set(Some("Sign in again to update inquiries.".to_owned()));
                    return;
                };

                let id = id.clone();
                let next_status = next_status.clone();
                let inquiries = inquiries.clone();
                let status = status.clone();

                status.set(Some("Updating inquiry...".to_owned()));
                spawn_local(async move {
                    match update_admin_inquiry_status(&token, &id, &next_status).await {
                        Ok(updated) => {
                            let mut next_items = (*inquiries).clone();
                            if let Some(existing) =
                                next_items.iter_mut().find(|item| item.id == updated.id)
                            {
                                *existing = updated;
                            }
                            inquiries.set(next_items);
                            status.set(None);
                        }
                        Err(error) => status.set(Some(error)),
                    }
                });
            })
        }
    };

    let delete_item = {
        let admin_token = admin_token.clone();
        let inquiries = inquiries.clone();
        let status = status.clone();

        move |id: String| {
            let token = (*admin_token).clone();
            let inquiries = inquiries.clone();
            let status = status.clone();

            Callback::from(move |_| {
                let Some(token) = token.clone() else {
                    status.set(Some("Sign in again to delete inquiries.".to_owned()));
                    return;
                };

                let id = id.clone();
                let inquiries = inquiries.clone();
                let status = status.clone();

                status.set(Some("Deleting inquiry...".to_owned()));
                spawn_local(async move {
                    match delete_admin_inquiry(&token, &id).await {
                        Ok(()) => {
                            let next_items = inquiries
                                .iter()
                                .filter(|item| item.id != id)
                                .cloned()
                                .collect::<Vec<_>>();
                            inquiries.set(next_items);
                            status.set(None);
                        }
                        Err(error) => status.set(Some(error)),
                    }
                });
            })
        }
    };

    html! {
        <main class="admin-shell">
            <section class="admin-panel">
                <div class="admin-heading">
                    <span class="brand-mark">{"C&Co"}</span>
                    <div>
                        <p class="eyebrow">{"Admin"}</p>
                        <h1>{"Contact requests"}</h1>
                    </div>
                </div>

                if admin_token.is_none() {
                    <form class="admin-login" onsubmit={onsubmit}>
                        <label>
                            {"Username"}
                            <input value={login.username.clone()} oninput={update_field("username")} autocomplete="username" required=true />
                        </label>
                        <label>
                            {"Password"}
                            <input r#type="password" value={login.password.clone()} oninput={update_field("password")} autocomplete="current-password" required=true />
                        </label>
                        <button class="button primary" type="submit">{"Sign in"}</button>
                        <p class="form-status">{status.as_ref().cloned().unwrap_or_default()}</p>
                    </form>
                } else {
                    <div class="admin-list">
                        <div class="admin-list-header">
                            <strong>{format!("{} contact request{}", inquiries.len(), if inquiries.len() == 1 { "" } else { "s" })}</strong>
                            <button class="button primary" type="button" onclick={onrefresh}>{"Refresh"}</button>
                        </div>
                        <p class="form-status">{status.as_ref().cloned().unwrap_or_default()}</p>
                        if inquiries.is_empty() {
                            <p class="empty-state">{"No contact requests yet."}</p>
                        } else {
                            {for inquiries.iter().map(|inquiry| html! {
                                <article class="inquiry-row">
                                    <div>
                                        <div class="inquiry-title">
                                            <h2>{inquiry.name.clone()}</h2>
                                            <span class={classes!("status-badge", status_class(&inquiry.status))}>{status_label(&inquiry.status)}</span>
                                        </div>
                                        <p>{inquiry.message.clone()}</p>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>{"Email"}</dt>
                                            <dd><a href={format!("mailto:{}", inquiry.email)}>{inquiry.email.clone()}</a></dd>
                                        </div>
                                        <div>
                                            <dt>{"Phone"}</dt>
                                            <dd>{empty_fallback(&inquiry.phone)}</dd>
                                        </div>
                                        <div>
                                            <dt>{"Pet"}</dt>
                                            <dd>{empty_fallback(&inquiry.pet_name)}</dd>
                                        </div>
                                    </dl>
                                    <div class="inquiry-actions">
                                        <button class="button secondary admin-action" type="button" onclick={update_status(inquiry.id.clone(), "submitted")}>{"Submitted"}</button>
                                        <button class="button secondary admin-action" type="button" onclick={update_status(inquiry.id.clone(), "contacted")}>{"Contacted"}</button>
                                        <button class="button secondary admin-action" type="button" onclick={update_status(inquiry.id.clone(), "purchased")}>{"Purchased"}</button>
                                        <button class="button danger admin-action" type="button" onclick={delete_item(inquiry.id.clone())}>{"Delete"}</button>
                                    </div>
                                </article>
                            })}
                        }
                    </div>
                }
            </section>
        </main>
    }
}

fn empty_fallback(value: &str) -> String {
    if value.trim().is_empty() {
        "Not provided".to_owned()
    } else {
        value.to_owned()
    }
}

fn status_label(status: &str) -> &'static str {
    match status {
        "contacted" => "Contacted",
        "purchased" => "Purchased",
        _ => "Submitted",
    }
}

fn status_class(status: &str) -> &'static str {
    match status {
        "contacted" => "status-contacted",
        "purchased" => "status-purchased",
        _ => "status-submitted",
    }
}

async fn admin_login(credentials: AdminLogin) -> Result<String, String> {
    let request = Request::post("/api/admin/login")
        .header("Content-Type", "application/json")
        .json(&credentials)
        .map_err(|error| format!("Could not prepare login: {error}"))?;

    match request.send().await {
        Ok(response) if response.ok() => response
            .json::<AdminLoginResponse>()
            .await
            .map(|login| login.token)
            .map_err(|error| format!("Could not read login response: {error}")),
        Ok(response) if response.status() == 401 => Err("Invalid username or password.".to_owned()),
        Ok(response) => Err(format!("Login failed with status {}.", response.status())),
        Err(error) => Err(format!("Could not sign in: {error}")),
    }
}

async fn fetch_admin_inquiries(token: &str) -> Result<Vec<Inquiry>, String> {
    match Request::get("/api/admin/inquiries")
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
    {
        Ok(response) if response.ok() => response
            .json::<Vec<Inquiry>>()
            .await
            .map_err(|error| format!("Could not read inquiries: {error}")),
        Ok(response) if response.status() == 401 => {
            Err("Sign in again to view inquiries.".to_owned())
        }
        Ok(response) => Err(format!(
            "Inquiry request failed with status {}.",
            response.status()
        )),
        Err(error) => Err(format!("Could not load inquiries: {error}")),
    }
}

async fn update_admin_inquiry_status(
    token: &str,
    id: &str,
    next_status: &str,
) -> Result<Inquiry, String> {
    let payload = InquiryStatusUpdate {
        status: next_status.to_owned(),
    };
    let request = Request::patch(&format!("/api/admin/inquiries/{id}/status"))
        .header("Authorization", &format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .json(&payload)
        .map_err(|error| format!("Could not prepare status update: {error}"))?;

    match request.send().await {
        Ok(response) if response.ok() => response
            .json::<Inquiry>()
            .await
            .map_err(|error| format!("Could not read updated inquiry: {error}")),
        Ok(response) if response.status() == 401 => {
            Err("Sign in again to update inquiries.".to_owned())
        }
        Ok(response) => Err(format!(
            "Status update failed with status {}.",
            response.status()
        )),
        Err(error) => Err(format!("Could not update inquiry: {error}")),
    }
}

async fn delete_admin_inquiry(token: &str, id: &str) -> Result<(), String> {
    match Request::delete(&format!("/api/admin/inquiries/{id}"))
        .header("Authorization", &format!("Bearer {token}"))
        .send()
        .await
    {
        Ok(response) if response.ok() => Ok(()),
        Ok(response) if response.status() == 401 => {
            Err("Sign in again to delete inquiries.".to_owned())
        }
        Ok(response) => Err(format!("Delete failed with status {}.", response.status())),
        Err(error) => Err(format!("Could not delete inquiry: {error}")),
    }
}
