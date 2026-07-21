use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Clone, Debug, Default, Serialize)]
struct InquiryForm {
    name: String,
    email: String,
    phone: String,
    preferred_contact_method: String,
    city_or_zip: String,
    pet_name: String,
    pet_age: String,
    service_of_interest: String,
    preferred_timeframe: String,
    message: String,
    consent_acknowledged: bool,
    website: String,
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
    preferred_contact_method: String,
    city_or_zip: String,
    pet_name: String,
    pet_age: String,
    service_of_interest: String,
    preferred_timeframe: String,
    message: String,
    status: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PublicRoute {
    Home,
    About,
    Services,
    DogTraining,
    PuppyTraining,
    GroupDogClasses,
    ServiceAreas,
    Resources,
    Resource(&'static str),
    Contact,
    Faq,
    Privacy,
    Accessibility,
    NotFound,
}

#[function_component(App)]
fn app() -> Html {
    if current_path().trim_end_matches('/') == "/admin" {
        html! { <AdminPage /> }
    } else {
        html! { <PublicPage route={public_route(&current_path())} /> }
    }
}

fn current_path() -> String {
    web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .unwrap_or_else(|| "/".to_owned())
}

fn public_route(path: &str) -> PublicRoute {
    match path.trim_end_matches('/') {
        "" | "/" => PublicRoute::Home,
        "/about" => PublicRoute::About,
        "/services" => PublicRoute::Services,
        "/services/dog-training" => PublicRoute::DogTraining,
        "/services/puppy-training" => PublicRoute::PuppyTraining,
        "/services/group-dog-classes" => PublicRoute::GroupDogClasses,
        "/service-areas" => PublicRoute::ServiceAreas,
        "/resources" => PublicRoute::Resources,
        "/contact" => PublicRoute::Contact,
        "/faq" => PublicRoute::Faq,
        "/privacy" => PublicRoute::Privacy,
        "/accessibility" => PublicRoute::Accessibility,
        other => other
            .strip_prefix("/resources/")
            .and_then(resource_title)
            .map(PublicRoute::Resource)
            .unwrap_or(PublicRoute::NotFound),
    }
}

#[function_component(PublicPage)]
fn public_page(props: &PublicPageProps) -> Html {
    set_page_title(props.route);
    html! {
        <>
            <a class="skip-link" href="#content">{"Skip to content"}</a>
            <Header />
            <main id="content">
                {match props.route {
                    PublicRoute::Home => home_page(),
                    PublicRoute::About => simple_page("About Cooper & Co.", "Cooper & Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville. Use the listed contact options to ask about dog training, puppy training, and group dog classes."),
                    PublicRoute::Services => services_page(),
                    PublicRoute::DogTraining => service_page("Dog training", "Dog training inquiries can cover leash manners, focus, everyday skills, and class fit."),
                    PublicRoute::PuppyTraining => service_page("Puppy training", "Puppy training inquiries focus on early manners, confidence, routines, and age-appropriate class readiness."),
                    PublicRoute::GroupDogClasses => service_page("Group dog classes", "Group dog class inquiries help determine class fit, preparation, and current availability."),
                    PublicRoute::ServiceAreas => service_areas_page(),
                    PublicRoute::Resources => resources_page(),
                    PublicRoute::Resource(title) => resource_article_page(title),
                    PublicRoute::Contact => html! { <ContactSection title="Contact Cooper & Co." /> },
                    PublicRoute::Faq => faq_page(),
                    PublicRoute::Privacy => simple_page("Privacy policy", "The inquiry form collects contact and pet-service details so Cooper & Co. can respond. Do not submit emergency, financial, or private medical information."),
                    PublicRoute::Accessibility => simple_page("Accessibility statement", "This site aims to provide semantic headings, keyboard-friendly navigation, visible focus states, and readable forms."),
                    PublicRoute::NotFound => simple_page("This Cooper & Co. page was not found", "Use the current service, resource, or contact links to continue."),
                }}
            </main>
            <Footer />
        </>
    }
}

#[derive(Clone, Copy, PartialEq, Properties)]
struct PublicPageProps {
    route: PublicRoute,
}

#[function_component(Header)]
fn header() -> Html {
    html! {
        <header class="topbar">
            <a class="brand" href="/" aria-label="Cooper and Co home">
                <span class="brand-mark" aria-hidden="true">{"C&Co"}</span>
                <span>{"Cooper & Co."}</span>
            </a>
            <nav aria-label="Main navigation">
                <a href="/about">{"About"}</a>
                <a href="/services">{"Services"}</a>
                <a href="/service-areas">{"Service areas"}</a>
                <a href="/resources">{"Resources"}</a>
                <a href="/faq">{"FAQ"}</a>
                <a href="/contact">{"Contact"}</a>
            </nav>
        </header>
    }
}

#[function_component(Footer)]
fn footer() -> Html {
    html! {
        <footer>
            <span>{"Cooper & Co. · Lorain, Ohio · "}<a href="tel:+14402761716">{"(440) 276-1716"}</a></span>
            <a href="/privacy">{"Privacy"}</a>
            <a href="/accessibility">{"Accessibility"}</a>
            <a href="https://www.facebook.com/CooperAndCoPet" rel="noreferrer">{"Facebook"}</a>
        </footer>
    }
}

fn home_page() -> Html {
    html! {
        <>
            <section class="hero" aria-labelledby="home-title">
                <picture class="hero-media">
                    <source srcset="/assets/cooperco-pet-services-hero.avif" type="image/avif" />
                    <img class="hero-image" src="/assets/cooperco-pet-services-hero.webp" alt="Black and tan dog on a leash in a park with dog-training cones in the background" width="1600" height="900" fetchpriority="high" decoding="async" />
                </picture>
                <div class="hero-copy">
                    <p class="eyebrow">{"Pet service based in Lorain County, Ohio"}</p>
                    <h1 id="home-title">{"Cooper & Co. dog training and pet services in Lorain County"}</h1>
                    <p>{"Cooper & Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville. Ask about dog training, puppy training, and group dog classes."}</p>
                    <div class="hero-actions">
                        <a class="button primary" href="/contact">{"Request information"}</a>
                        <a class="button secondary" href="tel:+14402761716">{"(440) 276-1716"}</a>
                    </div>
                </div>
            </section>
            <section class="section" aria-labelledby="home-services-title">
                <div class="section-heading">
                    <p class="eyebrow">{"Services"}</p>
                    <h2 id="home-services-title">{"Dog training and class inquiries"}</h2>
                    <p>{"Use the current service pages to share dog details, training goals, location, and preferred timing."}</p>
                </div>
                <div class="service-grid">
                    <article class="card"><h3><a href="/services/dog-training">{"Dog training"}</a></h3><p>{"Everyday training goals, leash manners, focus, and class-fit questions."}</p></article>
                    <article class="card"><h3><a href="/services/puppy-training">{"Puppy training"}</a></h3><p>{"Early manners, confidence building, routines, and puppy class preparation."}</p></article>
                    <article class="card"><h3><a href="/services/group-dog-classes">{"Group dog classes"}</a></h3><p>{"Class readiness, preparation, and current availability inquiries."}</p></article>
                </div>
            </section>
            <section class="section split" aria-labelledby="process-title">
                <div>
                    <p class="eyebrow">{"Process"}</p>
                    <h2 id="process-title">{"Start with a clear inquiry"}</h2>
                    <p>{"Share your city or ZIP code, pet age, goals, service interest, and preferred timeframe so Cooper & Co. can confirm fit."}</p>
                </div>
                <article class="update">
                    <span>{"Current next step"}</span>
                    <h3>{"Share your goals before booking"}</h3>
                    <p>{"Use the inquiry form to describe your dog, location, goals, and preferred timeframe."}</p>
                </article>
            </section>
            <ContactSection title="Ask about training or classes" />
        </>
    }
}

fn services_page() -> Html {
    html! {
        <section class="section" aria-labelledby="services-title">
            <div class="section-heading">
                <p class="eyebrow">{"Services"}</p>
                <h1 id="services-title">{"Dog training services from Cooper & Co."}</h1>
                <p>{"Use the current service pages to share dog details, training goals, location, and preferred timing."}</p>
            </div>
            <div class="service-grid">
                <article class="card"><h3><a href="/services/dog-training">{"Dog training"}</a></h3><p>{"Everyday training goals, leash manners, focus, and class-fit questions."}</p></article>
                <article class="card"><h3><a href="/services/puppy-training">{"Puppy training"}</a></h3><p>{"Early manners, confidence building, routines, and puppy class preparation."}</p></article>
                <article class="card"><h3><a href="/services/group-dog-classes">{"Group dog classes"}</a></h3><p>{"Class readiness, preparation, and current availability inquiries."}</p></article>
            </div>
        </section>
    }
}

fn service_page(title: &str, copy: &str) -> Html {
    html! {
        <>
            <section class="section page-hero" aria-labelledby="service-title">
                <p class="eyebrow">{"Service"}</p>
                <h1 id="service-title">{title}</h1>
                <p>{copy}</p>
                <div class="hero-actions">
                    <a class="button primary" href="/contact">{"Request information"}</a>
                    <a class="button secondary on-light" href="tel:+14402761716">{"(440) 276-1716"}</a>
                </div>
            </section>
            <section class="section" aria-labelledby="service-details">
                <div class="section-heading">
                    <p class="eyebrow">{"Details"}</p>
                    <h2 id="service-details">{"What to include"}</h2>
                    <p>{"Send your dog details, goals, city or ZIP code, health or safety notes, and scheduling preferences."}</p>
                </div>
            </section>
            <ContactSection title="Ask about this service" />
        </>
    }
}

fn service_areas_page() -> Html {
    html! {
        <section class="section page-hero" aria-labelledby="areas-title">
            <p class="eyebrow">{"Service areas"}</p>
            <h1 id="areas-title">{"Cooper & Co. service areas"}</h1>
            <p>{"Cooper & Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville."}</p>
            <div class="service-grid">
                <article class="card"><h3>{"Elyria, OH"}</h3><p>{"Include your city or ZIP code when sending an inquiry."}</p></article>
                <article class="card"><h3>{"Lorain, OH"}</h3><p>{"Include your city or ZIP code when sending an inquiry."}</p></article>
                <article class="card"><h3>{"Amherst, Avon, and North Ridgeville"}</h3><p>{"Ask directly about current availability in your city or ZIP code."}</p></article>
            </div>
        </section>
    }
}

fn resources_page() -> Html {
    html! {
        <section class="section page-hero" aria-labelledby="resources-title">
            <p class="eyebrow">{"Resources"}</p>
            <h1 id="resources-title">{"Dog training resources"}</h1>
            <p>{"Read practical training and class-preparation articles. Medical concerns should be directed to a qualified veterinarian."}</p>
            <div class="service-grid">
                <article class="card"><h3><a href="/resources/what-to-expect-from-a-group-dog-training-class">{"What to Expect From a Group Dog Training Class"}</a></h3><p>{"Class structure, preparation, and expectations."}</p></article>
                <article class="card"><h3><a href="/resources/preparing-your-puppy-for-its-first-training-class">{"Preparing Your Puppy for Its First Training Class"}</a></h3><p>{"Puppy comfort, packing, and health-aware planning."}</p></article>
                <article class="card"><h3><a href="/resources/basic-leash-skills-to-practice-at-home">{"Basic Leash Skills to Practice at Home"}</a></h3><p>{"Simple leash and focus ideas before asking about training."}</p></article>
            </div>
        </section>
    }
}

fn resource_title(slug: &str) -> Option<&'static str> {
    match slug {
        "what-to-expect-from-a-group-dog-training-class" => {
            Some("What to Expect From a Group Dog Training Class")
        }
        "preparing-your-puppy-for-its-first-training-class" => {
            Some("Preparing Your Puppy for Its First Training Class")
        }
        "basic-leash-skills-to-practice-at-home" => Some("Basic Leash Skills to Practice at Home"),
        "how-to-choose-a-dog-trainer-in-lorain-county" => {
            Some("How to Choose a Dog Trainer in Lorain County")
        }
        "questions-to-ask-before-joining-a-group-dog-class" => {
            Some("Questions to Ask Before Joining a Group Dog Class")
        }
        "puppy-socialization-without-overwhelming-your-puppy" => {
            Some("Puppy Socialization Without Overwhelming Your Puppy")
        }
        "helping-a-dog-stay-focused-around-distractions" => {
            Some("Helping a Dog Stay Focused Around Distractions")
        }
        "what-to-bring-to-a-dog-training-class" => Some("What to Bring to a Dog Training Class"),
        "dog-training-goals-how-to-set-realistic-expectations" => {
            Some("Dog Training Goals: How to Set Realistic Expectations")
        }
        "indoor-dog-enrichment-ideas-for-ohio-winters" => {
            Some("Indoor Dog-Enrichment Ideas for Ohio Winters")
        }
        _ => None,
    }
}

fn resource_article_page(title: &str) -> Html {
    html! {
        <>
            <article class="section page-hero resource-article" aria-labelledby="article-title">
                <p class="eyebrow">{"Cooper & Co. Resource"}</p>
                <h1 id="article-title">{title}</h1>
                <p>{"This resource helps dog owners prepare thoughtful questions before contacting Cooper & Co. For medical concerns, consult a qualified veterinarian."}</p>
                <div class="article-body">
                    <h2>{"Planning notes"}</h2>
                    <p>{"Use the article to think through your dog's age, comfort level, goals, safety notes, and what you want to practice."}</p>
                    <h2>{"Next step"}</h2>
                    <p>{"Share your city or ZIP code, service interest, and goals when you contact Cooper & Co."}</p>
                </div>
                <div class="hero-actions">
                    <a class="button primary" href="/contact">{"Contact Cooper & Co."}</a>
                    <a class="button secondary on-light" href="/resources">{"All resources"}</a>
                </div>
            </article>
        </>
    }
}

fn faq_page() -> Html {
    html! {
        <section class="section faq" aria-labelledby="faq-title">
            <div class="section-heading">
                <p class="eyebrow">{"FAQ"}</p>
                <h1 id="faq-title">{"Cooper & Co. questions"}</h1>
            </div>
            <details open=true><summary>{"Where does Cooper & Co. serve?"}</summary><p>{"Cooper & Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville."}</p></details>
            <details><summary>{"Which services are published?"}</summary><p>{"Dog training, puppy training, and group dog classes."}</p></details>
            <details><summary>{"Are hours or prices published?"}</summary><p>{"The website does not publish fixed hours or prices. Use the contact form, phone, or email for current details."}</p></details>
        </section>
    }
}

fn simple_page(title: &str, copy: &str) -> Html {
    html! {
        <section class="section page-hero" aria-labelledby="simple-title">
            <p class="eyebrow">{"Cooper & Co."}</p>
            <h1 id="simple-title">{title}</h1>
            <p>{copy}</p>
            <div class="hero-actions">
                <a class="button primary" href="/contact">{"Contact"}</a>
                <a class="button secondary on-light" href="/services">{"View services"}</a>
            </div>
        </section>
    }
}

#[derive(Clone, PartialEq, Properties)]
struct ContactProps {
    title: &'static str,
}

#[function_component(ContactSection)]
fn contact_section(props: &ContactProps) -> Html {
    let form = use_state(InquiryForm::default);
    let status = use_state(|| "idle".to_owned());

    let input_handler = {
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
                    "preferred_contact_method" => next.preferred_contact_method = value,
                    "city_or_zip" => next.city_or_zip = value,
                    "pet_name" => next.pet_name = value,
                    "pet_age" => next.pet_age = value,
                    "service_of_interest" => next.service_of_interest = value,
                    "preferred_timeframe" => next.preferred_timeframe = value,
                    "message" => next.message = value,
                    "website" => next.website = value,
                    _ => {}
                }
                form.set(next);
            })
        }
    };

    let change_handler = {
        let form = form.clone();
        move |field: &'static str| {
            let form = form.clone();
            Callback::from(move |event: Event| {
                let value = event
                    .target_dyn_into::<HtmlSelectElement>()
                    .map(|select| select.value())
                    .unwrap_or_default();
                let mut next = (*form).clone();
                match field {
                    "preferred_contact_method" => next.preferred_contact_method = value,
                    "service_of_interest" => next.service_of_interest = value,
                    _ => {}
                }
                form.set(next);
            })
        }
    };

    let consent_handler = {
        let form = form.clone();
        Callback::from(move |event: Event| {
            let checked = event
                .target_dyn_into::<HtmlInputElement>()
                .map(|input| input.checked())
                .unwrap_or(false);
            let mut next = (*form).clone();
            next.consent_acknowledged = checked;
            form.set(next);
        })
    };

    let onsubmit = {
        let form = form.clone();
        let status = status.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            if *status == "sending" {
                return;
            }
            let payload = (*form).clone();
            let form = form.clone();
            let status = status.clone();
            status.set("sending".to_owned());
            spawn_local(async move {
                let request = Request::post("/api/inquiries")
                    .header("Content-Type", "application/json")
                    .json(&payload);
                let Ok(request) = request else {
                    status.set("Could not prepare inquiry.".to_owned());
                    return;
                };
                match request.send().await {
                    Ok(response) if response.ok() => {
                        form.set(InquiryForm::default());
                        status.set("sent".to_owned());
                    }
                    Ok(response) => {
                        status.set(format!(
                            "Please check the form. Status {}.",
                            response.status()
                        ));
                    }
                    Err(error) => {
                        status.set(format!("Could not send inquiry: {error}"));
                    }
                }
            });
        })
    };

    html! {
        <section class="section contact" aria-labelledby="contact-title">
            <div class="contact-copy">
                <p class="eyebrow">{"Contact"}</p>
                <h2 id="contact-title">{props.title}</h2>
                <a href="mailto:cooper.copetservices@gmail.com">{"cooper.copetservices@gmail.com"}</a>
                <a href="tel:+14402761716">{"(440) 276-1716"}</a>
                <a href="https://www.facebook.com/CooperAndCoPet" rel="noreferrer">{"Facebook"}</a>
            </div>
            <form onsubmit={onsubmit} aria-label="Pet service inquiry form">
                <label for="name">{"Name"}<input id="name" name="name" autocomplete="name" value={form.name.clone()} oninput={input_handler("name")} required=true /></label>
                <label for="email">{"Email"}<input id="email" name="email" r#type="email" autocomplete="email" value={form.email.clone()} oninput={input_handler("email")} required=true /></label>
                <label for="phone">{"Phone"}<input id="phone" name="phone" r#type="tel" autocomplete="tel" value={form.phone.clone()} oninput={input_handler("phone")} /></label>
                <label for="preferred-contact">{"Preferred contact method"}<select id="preferred-contact" name="preferred_contact_method" value={form.preferred_contact_method.clone()} onchange={change_handler("preferred_contact_method")}><option value="">{"Choose one"}</option><option value="email">{"Email"}</option><option value="phone">{"Phone"}</option><option value="text">{"Text"}</option></select></label>
                <label for="city-or-zip">{"City or ZIP code"}<input id="city-or-zip" name="city_or_zip" autocomplete="postal-code" value={form.city_or_zip.clone()} oninput={input_handler("city_or_zip")} required=true /></label>
                <label for="pet-name">{"Pet name"}<input id="pet-name" name="pet_name" value={form.pet_name.clone()} oninput={input_handler("pet_name")} /></label>
                <label for="pet-age">{"Pet age"}<input id="pet-age" name="pet_age" value={form.pet_age.clone()} oninput={input_handler("pet_age")} /></label>
                <label for="service-interest">{"Service of interest"}<select id="service-interest" name="service_of_interest" value={form.service_of_interest.clone()} onchange={change_handler("service_of_interest")}><option value="">{"Choose one"}</option><option value="dog training">{"Dog training"}</option><option value="puppy training">{"Puppy training"}</option><option value="group dog classes">{"Group dog classes"}</option><option value="not sure">{"Not sure"}</option></select></label>
                <label for="timeframe">{"Preferred timeframe"}<input id="timeframe" name="preferred_timeframe" value={form.preferred_timeframe.clone()} oninput={input_handler("preferred_timeframe")} /></label>
                <label class="wide" for="message">{"Goals or needs"}<textarea id="message" name="message" value={form.message.clone()} oninput={input_handler("message")} required=true /></label>
                <label class="wide consent" for="consent"><input id="consent" name="consent_acknowledged" r#type="checkbox" checked={form.consent_acknowledged} onchange={consent_handler} required=true />{"I consent to Cooper & Co. using this information to respond to my inquiry."}</label>
                <label class="hp" for="website">{"Website"}<input id="website" name="website" tabindex="-1" value={form.website.clone()} oninput={input_handler("website")} /></label>
                <p class="privacy-note wide">{"Do not submit emergency, financial, or private medical information through this form."}</p>
                <button class="button primary" type="submit" disabled={*status == "sending"} aria-busy={(*status == "sending").to_string()}>{"Send inquiry"}</button>
                <p class="form-status" role="status" aria-live="polite">{match status.as_str() {
                    "idle" => "",
                    "sending" => "Sending...",
                    "sent" => "Inquiry sent.",
                    other => other,
                }}</p>
            </form>
        </section>
    }
}

fn set_page_title(route: PublicRoute) {
    let title = match route {
        PublicRoute::Home => "Cooper & Co. | Dog Training in Lorain County, Ohio",
        PublicRoute::About => "About Cooper & Co. in Lorain County",
        PublicRoute::Services => "Dog Training Services in Lorain County",
        PublicRoute::DogTraining => "Dog Training in Lorain County, Ohio | Cooper & Co.",
        PublicRoute::PuppyTraining => "Puppy Training in Lorain County | Cooper & Co.",
        PublicRoute::GroupDogClasses => "Group Dog Classes in Lorain County | Cooper & Co.",
        PublicRoute::ServiceAreas => "Service Areas in Lorain County | Cooper & Co.",
        PublicRoute::Resources => "Dog Training Resources | Cooper & Co.",
        PublicRoute::Resource(title) => title,
        PublicRoute::Contact => "Contact Cooper & Co. in Lorain County",
        PublicRoute::Faq => "FAQ | Cooper & Co.",
        PublicRoute::Privacy => "Privacy Policy | Cooper & Co.",
        PublicRoute::Accessibility => "Accessibility Statement | Cooper & Co.",
        PublicRoute::NotFound => "Page Not Found | Cooper & Co.",
    };
    if let Some(document) = web_sys::window().and_then(|window| window.document()) {
        document.set_title(title);
    }
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
    }

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
                            {for inquiries.iter().map(inquiry_row)}
                        }
                    </div>
                }
            </section>
        </main>
    }
}

fn inquiry_row(inquiry: &Inquiry) -> Html {
    html! {
        <article class="inquiry-row">
            <div>
                <div class="inquiry-title">
                    <h2>{inquiry.name.clone()}</h2>
                    <span class={classes!("status-badge", status_class(&inquiry.status))}>{status_label(&inquiry.status)}</span>
                </div>
                <p>{inquiry.message.clone()}</p>
            </div>
            <dl>
                <div><dt>{"Email"}</dt><dd><a href={format!("mailto:{}", inquiry.email)}>{inquiry.email.clone()}</a></dd></div>
                <div><dt>{"Phone"}</dt><dd>{empty_fallback(&inquiry.phone)}</dd></div>
                <div><dt>{"Preferred contact"}</dt><dd>{empty_fallback(&inquiry.preferred_contact_method)}</dd></div>
                <div><dt>{"City or ZIP"}</dt><dd>{empty_fallback(&inquiry.city_or_zip)}</dd></div>
                <div><dt>{"Pet"}</dt><dd>{format!("{} {}", empty_fallback(&inquiry.pet_name), empty_fallback(&inquiry.pet_age))}</dd></div>
                <div><dt>{"Service"}</dt><dd>{empty_fallback(&inquiry.service_of_interest)}</dd></div>
                <div><dt>{"Timeframe"}</dt><dd>{empty_fallback(&inquiry.preferred_timeframe)}</dd></div>
            </dl>
        </article>
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

fn main() {
    yew::Renderer::<App>::new().render();
}
