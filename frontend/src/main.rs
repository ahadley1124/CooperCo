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

#[derive(Clone, Debug, PartialEq, Deserialize)]
struct AdminProfile {
    email: String,
    name: Option<String>,
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

#[derive(Clone, Debug, Default, Serialize)]
struct InquiryForm {
    name: String,
    email: String,
    phone: String,
    pet_name: String,
    message: String,
}

#[derive(Clone, Debug, PartialEq, Properties)]
struct PublicPageProps {
    route: PublicRoute,
}

#[derive(Clone, Copy)]
struct SeoService {
    slug: &'static str,
    name: &'static str,
    summary: &'static str,
    title: &'static str,
}

#[derive(Clone, Copy)]
struct SeoLocation {
    slug: &'static str,
    name: &'static str,
    county_hint: &'static str,
}

#[derive(Clone, Copy)]
struct FaqItem {
    question: &'static str,
    answer: &'static str,
}

const SEO_SERVICES: &[SeoService] = &[
    SeoService {
        slug: "dog-walking",
        name: "Dog walking",
        summary: "Structured dog walking inquiry pages with schedule, leash, and neighborhood notes ready for owner-confirmed details.",
        title: "Dog Walking",
    },
    SeoService {
        slug: "dog-training",
        name: "Dog training",
        summary: "Dog training inquiry pages for goals, behavior notes, puppy basics, and current availability questions.",
        title: "Dog Training",
    },
    SeoService {
        slug: "pet-sitting",
        name: "Pet sitting",
        summary: "Pet sitting inquiry pages with feeding, medication, routine, and visit detail placeholders.",
        title: "Pet Sitting",
    },
    SeoService {
        slug: "house-sitting",
        name: "House sitting",
        summary: "House sitting inquiry pages for pet routines, home access, plant/mail notes, and overnight fit.",
        title: "House Sitting",
    },
    SeoService {
        slug: "puppy-care",
        name: "Puppy care",
        summary: "Puppy care inquiry pages for potty breaks, crate routines, socialization, and young-dog schedule needs.",
        title: "Puppy Care",
    },
    SeoService {
        slug: "dog-adventures",
        name: "Dog adventures",
        summary: "Dog adventure inquiry pages for enrichment walks, outings, transportation notes, and safety fit.",
        title: "Dog Adventures",
    },
];

const SEO_LOCATIONS: &[SeoLocation] = &[
    SeoLocation {
        slug: "mansfield-oh",
        name: "Mansfield, OH",
        county_hint: "Richland County",
    },
    SeoLocation {
        slug: "ontario-oh",
        name: "Ontario, OH",
        county_hint: "Richland County",
    },
    SeoLocation {
        slug: "lexington-oh",
        name: "Lexington, OH",
        county_hint: "Richland County",
    },
    SeoLocation {
        slug: "bellville-oh",
        name: "Bellville, OH",
        county_hint: "Richland County",
    },
    SeoLocation {
        slug: "ashland-oh",
        name: "Ashland, OH",
        county_hint: "Ashland County",
    },
    SeoLocation {
        slug: "galion-oh",
        name: "Galion, OH",
        county_hint: "Crawford/Richland County area",
    },
];

const SERVICE_FAQS: &[FaqItem] = &[
    FaqItem {
        question: "How do I confirm current availability?",
        answer: "Use the contact form or call Cooper & Co. with your city, pet details, dates, and service goals. TODO: owner should add current booking windows.",
    },
    FaqItem {
        question: "Are prices listed online?",
        answer: "Pricing is intentionally owner-confirmed because service length, travel, pet needs, and schedule affect fit.",
    },
    FaqItem {
        question: "What details should I include?",
        answer: "Share your dog's age, temperament, routine, location, desired dates, and any training or care concerns.",
    },
];

#[function_component(App)]
fn app() -> Html {
    match current_route() {
        AppRoute::Admin => html! { <AdminPage /> },
        AppRoute::Public(route) => html! { <PublicPage route={route} /> },
    }
}

enum AppRoute {
    Admin,
    Public(PublicRoute),
}

#[derive(Clone, Debug, PartialEq)]
enum PublicRoute {
    Home,
    Services,
    Service(String),
    Location(String),
    Contact,
    NotFound,
}

fn current_route() -> AppRoute {
    let path = web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .unwrap_or_default();

    if path.trim_end_matches('/') == "/admin" {
        AppRoute::Admin
    } else {
        AppRoute::Public(public_route(&path))
    }
}

fn public_route(path: &str) -> PublicRoute {
    let normalized = path.trim_end_matches('/');
    if normalized.is_empty() || normalized == "/" {
        PublicRoute::Home
    } else if normalized == "/services" {
        PublicRoute::Services
    } else if normalized == "/contact" {
        PublicRoute::Contact
    } else if let Some(slug) = normalized.strip_prefix("/services/") {
        PublicRoute::Service(slug.to_owned())
    } else if let Some(slug) = normalized.strip_prefix("/service-area/") {
        PublicRoute::Location(slug.to_owned())
    } else {
        PublicRoute::NotFound
    }
}

#[function_component(PublicPage)]
fn public_page(props: &PublicPageProps) -> Html {
    let content = use_state(|| Some(seed_content()));
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

    set_page_title(&props.route);

    match &props.route {
        PublicRoute::Service(slug) => {
            if let Some(service) = find_service(slug) {
                return service_page(&site, service);
            }
        }
        PublicRoute::Location(slug) => {
            if let Some(location) = find_location(slug) {
                return location_page(&site, location);
            }
        }
        PublicRoute::Services => return services_index_page(&site),
        PublicRoute::Contact => {}
        PublicRoute::NotFound => return not_found_page(&site),
        PublicRoute::Home => {}
    }

    html! {
        <>
            <a class="skip-link" href="#contact">{"Skip to contact"}</a>
            <header class="topbar">
                <a class="brand" href="/" aria-label="Cooper and Co home">
                    <span class="brand-mark" aria-hidden="true">{"C&Co"}</span>
                    <span>{site.business.name.clone()}</span>
                </a>
                <nav aria-label="Main navigation">
                    <a href="/services">{"Services"}</a>
                    <a href="/services/dog-training">{"Dog training"}</a>
                    <a href="/service-area/mansfield-oh">{"Service areas"}</a>
                    <a href="#faq">{"FAQ"}</a>
                    <a href="/contact">{"Contact"}</a>
                </nav>
            </header>

            <main>
            <section id="top" class="hero" aria-labelledby="home-title">
                <picture class="hero-media">
                    <source srcset={avif_src(&site.business.hero_image)} type="image/avif" />
                    <img class="hero-image" src={site.business.hero_image.clone()} alt="Cooper & Co. pet services logo from the public Facebook page" width="1600" height="900" sizes="100vw" fetchpriority="high" decoding="async" />
                </picture>
                <div class="hero-copy">
                    <p class="eyebrow">{format!("{} in {}", site.business.category, site.business.location)}</p>
                    <h1 id="home-title">{"Cooper & Co. pet services and dog training support"}</h1>
                    <p>{site.business.intro.clone()}</p>
                    <div class="hero-actions">
                        <a class="button primary" href="/contact">{"Request information"}</a>
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

            <section id="services" class="section" aria-labelledby="services-title">
                <div class="section-heading">
                    <p class="eyebrow">{"Services"}</p>
                    <h2 id="services-title">{"Pet services for Lorain County families"}</h2>
                </div>
                <div class="service-grid">
                    {for site.services.iter().map(|service| html! {
                        <article class="card">
                            <h3>{service.title.clone()}</h3>
                            <p>{service.summary.clone()}</p>
                            <a href="/services">{"Explore service pages"}</a>
                        </article>
                    })}
                </div>
            </section>

            <section id="group-classes" class="section split" aria-labelledby="classes-title">
                <div>
                    <p class="eyebrow">{"Group classes"}</p>
                    <h2 id="classes-title">{"Dog training and group classes in Lorain County"}</h2>
                    <p>{"Cooper & Co. shares class updates publicly and handles booking questions directly by phone, email, Facebook, and the contact form."}</p>
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
                    <picture>
                        <source srcset={avif_src(&image.src)} type="image/avif" />
                        <img src={image.src.clone()} alt={image.alt.clone()} width="1200" height="1200" loading="lazy" decoding="async" sizes="(max-width: 820px) 50vw, 33vw" />
                    </picture>
                })}
            </section>

            <section id="service-area" class="section" aria-labelledby="area-title">
                <div class="section-heading">
                    <p class="eyebrow">{"Service area"}</p>
                    <h2 id="area-title">{"Dog services near you"}</h2>
                    <p>{"These service-area pages support local search around Mansfield, Ontario, Lexington, Bellville, Ashland, and Galion. TODO: owner should confirm active coverage for each city."}</p>
                </div>
                <div class="service-grid">
                    {for SEO_LOCATIONS.iter().map(|location| html! {
                        <article id={location.slug} class="card"><h3><a href={format!("/service-area/{}", location.slug)}>{location.name}</a></h3><p>{format!("Dog service inquiry page for {} in {}.", location.name, location.county_hint)}</p></article>
                    })}
                </div>
            </section>

            <section id="faq" class="section faq" aria-labelledby="faq-title">
                <div class="section-heading">
                    <p class="eyebrow">{"Questions"}</p>
                    <h2 id="faq-title">{"Dog training, pricing, availability, and service area FAQ"}</h2>
                </div>
                <details open=true>
                    <summary>{"Does Cooper & Co. offer dog training in Lorain County?"}</summary>
                    <p>{"Yes. Cooper & Co. supports dog training and group class inquiries for Lorain County pet families."}</p>
                </details>
                <details>
                    <summary>{"Are group dog classes or puppy classes available now?"}</summary>
                    <p>{"Availability can change by season. Use the contact form, phone number, or Facebook page for current class openings."}</p>
                </details>
                <details>
                    <summary>{"How much do pet services or classes cost?"}</summary>
                    <p>{"Pricing depends on the service, class, and current availability. Contact Cooper & Co. with your pet details for current pricing."}</p>
                </details>
                <details>
                    <summary>{"What cities are in the Cooper & Co. service area?"}</summary>
                    <p>{"The service area centers on Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville, Ohio."}</p>
                </details>
            </section>

            <section id="contact" class="section contact" aria-labelledby="contact-title">
                <div class="contact-copy">
                    <p class="eyebrow">{"Contact"}</p>
                    <h2 id="contact-title">{"Ask about classes or pet support"}</h2>
                    <a href={format!("mailto:{}", site.business.email)}>{site.business.email.clone()}</a>
                    <a href={format!("tel:{}", site.business.phone.replace([' ', '(', ')', '-'], ""))}>{site.business.phone.clone()}</a>
                    <a href={site.business.facebook_url.clone()} target="_blank" rel="noreferrer">{"Facebook"}</a>
                    <a href={site.business.yelp_url.clone()} target="_blank" rel="noreferrer">{"Yelp listing"}</a>
                </div>
                <form onsubmit={onsubmit} aria-label="Pet service inquiry form">
                    <label for="name">
                        {"Name"}
                        <input id="name" name="name" autocomplete="name" value={form.name.clone()} oninput={update_field("name")} required=true aria-required="true" />
                    </label>
                    <label for="email">
                        {"Email"}
                        <input id="email" name="email" r#type="email" autocomplete="email" value={form.email.clone()} oninput={update_field("email")} required=true aria-required="true" />
                    </label>
                    <label for="phone">
                        {"Phone"}
                        <input id="phone" name="phone" r#type="tel" autocomplete="tel" value={form.phone.clone()} oninput={update_field("phone")} />
                    </label>
                    <label for="pet-name">
                        {"Pet name"}
                        <input id="pet-name" name="pet_name" value={form.pet_name.clone()} oninput={update_field("pet_name")} />
                    </label>
                    <label class="wide" for="message">
                        {"Message"}
                        <textarea id="message" name="message" value={form.message.clone()} oninput={update_field("message")} required=true aria-required="true" />
                    </label>
                    <button class="button primary" type="submit" disabled={*submit_state == "sending"} aria-busy={(*submit_state == "sending").to_string()}>{"Send inquiry"}</button>
                    <p class="form-status" role="status" aria-live="polite">{match submit_state.as_str() {
                        "idle" => "",
                        "sending" => "Sending...",
                        "sent" => "Inquiry sent.",
                        other => other,
                    }}</p>
                </form>
            </section>
            </main>

            <footer>
                <span>{format!("{} · {}", site.business.name, site.business.location)}</span>
                <a href="#services">{"Services"}</a>
                <a href="#contact">{"Contact"}</a>
                <a href={site.business.facebook_url} target="_blank" rel="noreferrer">{"Facebook"}</a>
            </footer>
        </>
    }
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

fn avif_src(src: &str) -> String {
    src.strip_suffix(".webp")
        .map(|base| format!("{base}.avif"))
        .unwrap_or_else(|| src.to_owned())
}

fn find_service(slug: &str) -> Option<&'static SeoService> {
    SEO_SERVICES.iter().find(|service| service.slug == slug)
}

fn find_location(slug: &str) -> Option<&'static SeoLocation> {
    SEO_LOCATIONS.iter().find(|location| location.slug == slug)
}

fn set_page_title(route: &PublicRoute) {
    let title = match route {
        PublicRoute::Service(slug) => find_service(slug)
            .map(|service| format!("{} | Cooper & Co.", service.title))
            .unwrap_or_else(|| "Cooper & Co. Dog Services".to_owned()),
        PublicRoute::Location(slug) => find_location(slug)
            .map(|location| format!("Dog Services in {} | Cooper & Co.", location.name))
            .unwrap_or_else(|| "Cooper & Co. Dog Services".to_owned()),
        PublicRoute::Services => {
            "Dog Walking, Training, Sitting & Puppy Care | Cooper & Co.".to_owned()
        }
        PublicRoute::Contact => "Contact Cooper & Co. Dog Services".to_owned(),
        PublicRoute::NotFound => "Page Not Found | Cooper & Co.".to_owned(),
        PublicRoute::Home => "Cooper & Co. Dog Services | Mansfield-Area Ohio".to_owned(),
    };

    if let Some(document) = web_sys::window().and_then(|window| window.document()) {
        document.set_title(&title);
    }
}

fn page_shell(site: &SiteContent, body: Html) -> Html {
    html! {
        <>
            <a class="skip-link" href="#content">{"Skip to content"}</a>
            <header class="topbar">
                <a class="brand" href="/" aria-label="Cooper and Co home">
                    <span class="brand-mark" aria-hidden="true">{"C&Co"}</span>
                    <span>{site.business.name.clone()}</span>
                </a>
                <nav aria-label="Main navigation">
                    <a href="/services">{"Services"}</a>
                    <a href="/services/dog-walking">{"Dog walking"}</a>
                    <a href="/services/dog-training">{"Dog training"}</a>
                    <a href="/service-area/mansfield-oh">{"Service areas"}</a>
                    <a href="/contact">{"Contact"}</a>
                </nav>
            </header>
            <main id="content">{body}</main>
            <footer>
                <span>{format!("{} · {}", site.business.name, site.business.location)}</span>
                <a href="/services">{"Services"}</a>
                <a href="/contact">{"Contact"}</a>
                <a href={site.business.facebook_url.clone()} target="_blank" rel="noreferrer">{"Facebook"}</a>
            </footer>
        </>
    }
}

fn services_index_page(site: &SiteContent) -> Html {
    page_shell(
        site,
        html! {
            <>
                <section class="section page-hero" aria-labelledby="services-page-title">
                    <p class="eyebrow">{"Dog services"}</p>
                    <h1 id="services-page-title">{"Dog walking, training, sitting, puppy care, and adventures"}</h1>
                    <p>{"Reusable service pages are ready for owner-confirmed descriptions, pricing, photos, and availability. Current copy avoids unverified claims."}</p>
                    <div class="hero-actions">
                        <a class="button primary" href="/contact">{"Book now"}</a>
                        <a class="button secondary on-light" href="tel:+14402761716">{"Call now"}</a>
                    </div>
                </section>
                <section class="section" aria-labelledby="service-list-title">
                    <div class="section-heading">
                        <p class="eyebrow">{"Services"}</p>
                        <h2 id="service-list-title">{"Choose a dog service page"}</h2>
                    </div>
                    <div class="service-grid">
                        {for SEO_SERVICES.iter().map(service_card)}
                    </div>
                </section>
                {faq_section("Service FAQ", SERVICE_FAQS)}
                {internal_links_section()}
            </>
        },
    )
}

fn service_page(site: &SiteContent, service: &SeoService) -> Html {
    page_shell(
        site,
        html! {
            <>
                <section class="section page-hero" aria-labelledby="service-title">
                    <p class="eyebrow">{"Dog service"}</p>
                    <h1 id="service-title">{format!("{} around Mansfield, Ontario, Lexington, Bellville, Ashland, and Galion", service.title)}</h1>
                    <p>{service.summary}</p>
                    <p class="todo-note">{"TODO owner input: add confirmed service description, pricing model, visit length, requirements, and current availability."}</p>
                    <div class="hero-actions">
                        <a class="button primary" href="/contact">{"Book now"}</a>
                        <a class="button secondary on-light" href="tel:+14402761716">{"Call now"}</a>
                    </div>
                </section>
                <section class="section" aria-labelledby="service-area-links-title">
                    <div class="section-heading">
                        <p class="eyebrow">{"Service areas"}</p>
                        <h2 id="service-area-links-title">{format!("{} by city", service.title)}</h2>
                        <p>{"Each city page links back to this service so local pages can grow without duplicated owner facts."}</p>
                    </div>
                    <div class="service-grid">
                        {for SEO_LOCATIONS.iter().map(|location| html! {
                            <article class="card">
                                <h3><a href={format!("/service-area/{}", location.slug)}>{format!("{} in {}", service.title, location.name)}</a></h3>
                                <p>{format!("Local inquiry page for {} in {}. TODO: owner should confirm active coverage and travel limits.", service.name.to_lowercase(), location.name)}</p>
                            </article>
                        })}
                    </div>
                </section>
                {faq_section("Service FAQ", SERVICE_FAQS)}
                {internal_links_section()}
            </>
        },
    )
}

fn location_page(site: &SiteContent, location: &SeoLocation) -> Html {
    page_shell(
        site,
        html! {
            <>
                <section class="section page-hero" aria-labelledby="location-title">
                    <p class="eyebrow">{"Service area"}</p>
                    <h1 id="location-title">{format!("Dog services in {}", location.name)}</h1>
                    <p>{format!("Dog walking, dog training, pet sitting, house sitting, puppy care, and dog adventure inquiry page for {} in {}.", location.name, location.county_hint)}</p>
                    <p class="todo-note">{"TODO owner input: confirm this city is actively served, add neighborhoods, landmarks, local photos, and any travel fees."}</p>
                    <div class="hero-actions">
                        <a class="button primary" href="/contact">{"Contact"}</a>
                        <a class="button secondary on-light" href="/services">{"View services"}</a>
                    </div>
                </section>
                <section class="section" aria-labelledby="location-services-title">
                    <div class="section-heading">
                        <p class="eyebrow">{"Available page templates"}</p>
                        <h2 id="location-services-title">{format!("Dog service pages for {}", location.name)}</h2>
                    </div>
                    <div class="service-grid">
                        {for SEO_SERVICES.iter().map(service_card)}
                    </div>
                </section>
                {faq_section("Local service FAQ", SERVICE_FAQS)}
                {internal_links_section()}
            </>
        },
    )
}

fn not_found_page(site: &SiteContent) -> Html {
    page_shell(
        site,
        html! {
            <section class="section page-hero" aria-labelledby="not-found-title">
                <p class="eyebrow">{"Not found"}</p>
                <h1 id="not-found-title">{"This Cooper & Co. page is not available"}</h1>
                <p>{"Use the service, service-area, resource, or contact links to continue."}</p>
                <div class="hero-actions">
                    <a class="button primary" href="/services">{"View services"}</a>
                    <a class="button secondary on-light" href="/contact">{"Contact"}</a>
                </div>
            </section>
        },
    )
}

fn service_card(service: &SeoService) -> Html {
    html! {
        <article class="card">
            <h3><a href={format!("/services/{}", service.slug)}>{service.name}</a></h3>
            <p>{service.summary}</p>
            <a href={format!("/services/{}", service.slug)}>{"Open service page"}</a>
        </article>
    }
}

fn faq_section(title: &str, items: &[FaqItem]) -> Html {
    html! {
        <section class="section faq" aria-labelledby="page-faq-title">
            <div class="section-heading">
                <p class="eyebrow">{"Questions"}</p>
                <h2 id="page-faq-title">{title}</h2>
            </div>
            {for items.iter().enumerate().map(|(index, item)| html! {
                <details open={index == 0}>
                    <summary>{item.question}</summary>
                    <p>{item.answer}</p>
                </details>
            })}
        </section>
    }
}

fn internal_links_section() -> Html {
    html! {
        <section class="section link-panel" aria-labelledby="internal-links-title">
            <div class="section-heading">
                <p class="eyebrow">{"Explore"}</p>
                <h2 id="internal-links-title">{"Service, location, resource, and contact links"}</h2>
            </div>
            <div class="link-columns">
                <div>
                    <h3>{"Services"}</h3>
                    {for SEO_SERVICES.iter().map(|service| html! { <a href={format!("/services/{}", service.slug)}>{service.name}</a> })}
                </div>
                <div>
                    <h3>{"Locations"}</h3>
                    {for SEO_LOCATIONS.iter().map(|location| html! { <a href={format!("/service-area/{}", location.slug)}>{location.name}</a> })}
                </div>
                <div>
                    <h3>{"Next steps"}</h3>
                    <a href="/resources">{"Resources"}</a>
                    <a href="/contact">{"Contact"}</a>
                    <a href="tel:+14402761716">{"Call Cooper & Co."}</a>
                </div>
            </div>
        </section>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(AdminPage)]
fn admin_page() -> Html {
    let profile = use_state(|| None::<AdminProfile>);
    let inquiries = use_state(Vec::<Inquiry>::new);
    let status = use_state(|| Some("Checking Microsoft session...".to_owned()));
    let signed_in = use_state(|| false);

    {
        let profile = profile.clone();
        let inquiries = inquiries.clone();
        let status = status.clone();
        let signed_in = signed_in.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match fetch_admin_profile().await {
                    Ok(admin) => {
                        profile.set(Some(admin));
                        signed_in.set(true);
                    }
                    Err(error) if error.starts_with("Sign in") => {
                        status.set(Some(error));
                        return;
                    }
                    Err(error) => {
                        status.set(Some(error));
                        return;
                    }
                }

                match fetch_admin_inquiries().await {
                    Ok(items) => {
                        inquiries.set(items);
                        status.set(None);
                    }
                    Err(error) => status.set(Some(error)),
                }
            });
            || ()
        });
    };

    let onrefresh = {
        let inquiries = inquiries.clone();
        let status = status.clone();

        Callback::from(move |_| {
            let inquiries = inquiries.clone();
            let status = status.clone();

            status.set(Some("Refreshing inquiries...".to_owned()));
            spawn_local(async move {
                match fetch_admin_inquiries().await {
                    Ok(items) => {
                        inquiries.set(items);
                        status.set(None);
                    }
                    Err(error) => status.set(Some(error)),
                }
            });
        })
    };

    let update_status = {
        let inquiries = inquiries.clone();
        let status = status.clone();

        move |id: String, next_status: &'static str| {
            let inquiries = inquiries.clone();
            let status = status.clone();
            let next_status = next_status.to_owned();

            Callback::from(move |_| {
                let id = id.clone();
                let next_status = next_status.clone();
                let inquiries = inquiries.clone();
                let status = status.clone();

                status.set(Some("Updating inquiry...".to_owned()));
                spawn_local(async move {
                    match update_admin_inquiry_status(&id, &next_status).await {
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
        let inquiries = inquiries.clone();
        let status = status.clone();

        move |id: String| {
            let inquiries = inquiries.clone();
            let status = status.clone();

            Callback::from(move |_| {
                let id = id.clone();
                let inquiries = inquiries.clone();
                let status = status.clone();

                status.set(Some("Deleting inquiry...".to_owned()));
                spawn_local(async move {
                    match delete_admin_inquiry(&id).await {
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
                        <p>{profile.as_ref().map(|admin| admin.email.clone()).unwrap_or_default()}</p>
                    </div>
                    if *signed_in {
                        <form action="/auth/logout" method="post">
                            <button class="button secondary" type="submit">{"Sign out"}</button>
                        </form>
                    }
                </div>

                if !*signed_in {
                    <div class="admin-login">
                        <p>{"Sign in with a permitted Microsoft account to manage Cooper & Co. inquiries."}</p>
                        <a class="button primary" href="/auth/microsoft/login">{"Sign in with Microsoft"}</a>
                        <p class="form-status">{status.as_ref().cloned().unwrap_or_default()}</p>
                    </div>
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

async fn fetch_admin_profile() -> Result<AdminProfile, String> {
    match Request::get("/api/admin/me").send().await {
        Ok(response) if response.ok() => response
            .json::<AdminProfile>()
            .await
            .map_err(|error| format!("Could not read admin profile: {error}")),
        Ok(response) if response.status() == 401 => {
            Err("Sign in with Microsoft to view admin inquiries.".to_owned())
        }
        Ok(response) => Err(format!(
            "Admin check failed with status {}.",
            response.status()
        )),
        Err(error) => Err(format!("Could not check admin session: {error}")),
    }
}

async fn fetch_admin_inquiries() -> Result<Vec<Inquiry>, String> {
    match Request::get("/api/admin/inquiries").send().await {
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

async fn update_admin_inquiry_status(id: &str, next_status: &str) -> Result<Inquiry, String> {
    let payload = InquiryStatusUpdate {
        status: next_status.to_owned(),
    };
    let request = Request::patch(&format!("/api/admin/inquiries/{id}/status"))
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

async fn delete_admin_inquiry(id: &str) -> Result<(), String> {
    match Request::delete(&format!("/api/admin/inquiries/{id}"))
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
