use std::{env, path::PathBuf};

use rocket::{
    fs::NamedFile,
    get,
    http::{ContentType, Header, Status},
    request::Request,
    response::{self, content::RawXml, Redirect, Responder, Response},
};
use serde_json::{json, Value};

const PRODUCTION_ORIGIN: &str = "https://cooper-and-co.com";
const LASTMOD: &str = "2026-07-19";
const SOCIAL_IMAGE: &str = "/assets/cooperco-pet-services-hero.webp";
const SOCIAL_IMAGE_ALT: &str = "Cooper & Co. dog training and pet service branding";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum LocationStatus {
    Confirmed,
    Unconfirmed,
    NotServed,
}

#[derive(Clone, Copy, Debug)]
pub struct BusinessProfile {
    pub name: &'static str,
    pub phone: &'static str,
    pub phone_e164: &'static str,
    pub email: &'static str,
    pub home_city: &'static str,
    pub state: &'static str,
    pub county: &'static str,
    pub public_address: Option<&'static str>,
    pub facebook_url: &'static str,
    pub yelp_url: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct ServiceDefinition {
    pub slug: &'static str,
    pub name: &'static str,
    pub page_title: &'static str,
    pub description: &'static str,
    pub summary: &'static str,
    pub audience: &'static str,
    pub process: &'static [&'static str],
    pub prepare: &'static [&'static str],
    pub faq: &'static [FaqItem],
    pub related_resources: &'static [&'static str],
}

#[derive(Clone, Copy, Debug)]
pub struct ServiceArea {
    pub slug: &'static str,
    pub name: &'static str,
    pub status: LocationStatus,
}

#[derive(Clone, Copy, Debug)]
pub struct FaqItem {
    pub question: &'static str,
    pub answer: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceArticle {
    pub slug: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub service_slug: &'static str,
    pub published: &'static str,
    pub modified: &'static str,
    pub sections: &'static [ArticleSection],
}

#[derive(Clone, Copy, Debug)]
pub struct ArticleSection {
    pub heading: &'static str,
    pub body: &'static str,
}

#[derive(Clone, Debug)]
pub struct Page {
    pub path: String,
    pub title: String,
    pub description: String,
    pub h1: String,
    pub body: String,
    pub breadcrumbs: Vec<(&'static str, String)>,
    pub schema: Vec<Value>,
    pub indexable: bool,
}

pub const BUSINESS: BusinessProfile = BusinessProfile {
    name: "Cooper & Co.",
    phone: "(440) 276-1716",
    phone_e164: "+14402761716",
    email: "cooper.copetservices@gmail.com",
    home_city: "Lorain",
    state: "Ohio",
    county: "Lorain County",
    public_address: None,
    facebook_url: "https://www.facebook.com/CooperAndCoPet",
    yelp_url: "https://m.yelp.com/biz/cooper-and-company-elyria",
};

pub const SERVICE_AREAS: &[ServiceArea] = &[
    ServiceArea {
        slug: "lorain-oh",
        name: "Lorain, OH",
        status: LocationStatus::Confirmed,
    },
    ServiceArea {
        slug: "elyria-oh",
        name: "Elyria, OH",
        status: LocationStatus::Unconfirmed,
    },
    ServiceArea {
        slug: "amherst-oh",
        name: "Amherst, OH",
        status: LocationStatus::Unconfirmed,
    },
    ServiceArea {
        slug: "sheffield-lake-oh",
        name: "Sheffield Lake, OH",
        status: LocationStatus::Unconfirmed,
    },
    ServiceArea {
        slug: "sheffield-village-oh",
        name: "Sheffield Village, OH",
        status: LocationStatus::Unconfirmed,
    },
    ServiceArea {
        slug: "avon-oh",
        name: "Avon, OH",
        status: LocationStatus::Unconfirmed,
    },
    ServiceArea {
        slug: "avon-lake-oh",
        name: "Avon Lake, OH",
        status: LocationStatus::Unconfirmed,
    },
    ServiceArea {
        slug: "north-ridgeville-oh",
        name: "North Ridgeville, OH",
        status: LocationStatus::Unconfirmed,
    },
    ServiceArea {
        slug: "oberlin-oh",
        name: "Oberlin, OH",
        status: LocationStatus::Unconfirmed,
    },
    ServiceArea {
        slug: "vermilion-oh",
        name: "Vermilion, OH",
        status: LocationStatus::Unconfirmed,
    },
];

const SERVICE_FAQ: &[FaqItem] = &[
    FaqItem {
        question: "How do I know which service is a good fit?",
        answer: "Share your dog's age, current skills, goals, schedule needs, and location. Cooper & Co. can respond with the most relevant next step.",
    },
    FaqItem {
        question: "Are prices listed online?",
        answer: "Pricing is owner-confirmed during inquiry because the website does not yet have approved pricing rules.",
    },
    FaqItem {
        question: "Can Cooper & Co. help with medical concerns?",
        answer: "Medical questions should go to a qualified veterinarian. Training and class inquiries should describe any health limits that affect participation.",
    },
];

pub const SERVICES: &[ServiceDefinition] = &[
    ServiceDefinition {
        slug: "dog-training",
        name: "Dog training",
        page_title: "Dog Training in Lorain, Ohio | Cooper & Co.",
        description: "Ask Cooper & Co. about dog training in Lorain, Ohio, including goals, current skills, class fit, and next steps for local pet families.",
        summary: "Dog training inquiries can cover leash manners, focus, everyday skills, and current training goals.",
        audience: "Appropriate for dog owners who want clearer expectations, practical skills, and help choosing a class or training path.",
        process: &[
            "Send an inquiry with your city, dog details, and training goals.",
            "Cooper & Co. reviews fit, schedule, and the safest next step.",
            "You receive current availability and preparation details directly from the business.",
        ],
        prepare: &[
            "Dog age, breed or size, and current training experience.",
            "Goals such as leash skills, focus, household manners, or class readiness.",
            "Known triggers, safety notes, health limits, and veterinarian guidance when relevant.",
        ],
        faq: SERVICE_FAQ,
        related_resources: &[
            "basic-leash-skills-to-practice-at-home",
            "helping-a-dog-stay-focused-around-distractions",
            "dog-training-goals-how-to-set-realistic-expectations",
        ],
    },
    ServiceDefinition {
        slug: "puppy-training",
        name: "Puppy training",
        page_title: "Puppy Training in Lorain County | Cooper & Co.",
        description: "Ask about puppy training and age-appropriate class fit with Cooper & Co., based in Lorain and serving local pet owners.",
        summary: "Puppy training inquiries focus on early manners, confidence, routines, and class readiness.",
        audience: "Appropriate for puppy owners who want early guidance without overwhelming a young dog.",
        process: &[
            "Share puppy age, vaccination status as approved by the owner, and goals.",
            "Cooper & Co. confirms whether the current format is a fit.",
            "You receive current preparation details before attending a class or session.",
        ],
        prepare: &[
            "Puppy age, schedule, comfort around people or dogs, and handling notes.",
            "Questions about potty routines, crate practice, leash exposure, or focus.",
            "Veterinarian guidance for medical or vaccination concerns.",
        ],
        faq: SERVICE_FAQ,
        related_resources: &[
            "preparing-your-puppy-for-its-first-training-class",
            "puppy-socialization-without-overwhelming-your-puppy",
            "what-to-bring-to-a-dog-training-class",
        ],
    },
    ServiceDefinition {
        slug: "group-dog-classes",
        name: "Group dog classes",
        page_title: "Group Dog Classes Near Lorain, OH | Cooper & Co.",
        description: "Learn how to ask Cooper & Co. about group dog classes near Lorain, Ohio, including class fit, preparation, and availability.",
        summary: "Group dog class inquiries help determine class fit, readiness, goals, and current openings.",
        audience: "Appropriate for owners who want structured practice around other dogs and people when group settings are a fit.",
        process: &[
            "Describe your dog's age, temperament, and goals before class.",
            "Cooper & Co. confirms whether the current group format is appropriate.",
            "You receive class preparation notes and availability from the business.",
        ],
        prepare: &[
            "Leash, collar or harness details, treats or rewards, and handling notes.",
            "Known reactivity, fear, overexcitement, or safety concerns.",
            "Questions about current class format, capacity, and requirements.",
        ],
        faq: SERVICE_FAQ,
        related_resources: &[
            "what-to-expect-from-a-group-dog-training-class",
            "questions-to-ask-before-joining-a-group-dog-class",
            "what-to-bring-to-a-dog-training-class",
        ],
    },
];

pub const ARTICLES: &[ResourceArticle] = &[
    ResourceArticle {
        slug: "what-to-expect-from-a-group-dog-training-class",
        title: "What to Expect From a Group Dog Training Class",
        description: "A practical overview of group dog class structure, preparation, and realistic training expectations.",
        service_slug: "group-dog-classes",
        published: "2026-07-19",
        modified: "2026-07-19",
        sections: &[
            ArticleSection { heading: "Class flow", body: "Group classes commonly include introductions, short skill demonstrations, practice time, and breaks so dogs can reset." },
            ArticleSection { heading: "Good preparation", body: "Bring questions about your dog's age, comfort level, leash skills, and what your household wants to practice between classes." },
            ArticleSection { heading: "Realistic outcomes", body: "Progress depends on practice, fit, health, and consistency. Avoid guarantees and ask for the safest next step for your dog." },
        ],
    },
    ResourceArticle {
        slug: "preparing-your-puppy-for-its-first-training-class",
        title: "Preparing Your Puppy for Its First Training Class",
        description: "Help your puppy arrive ready for a first training class with simple planning and health-aware questions.",
        service_slug: "puppy-training",
        published: "2026-07-19",
        modified: "2026-07-19",
        sections: &[
            ArticleSection { heading: "Start with comfort", body: "Practice short car rides, gentle handling, and quiet observation before asking a puppy to work in a busy class setting." },
            ArticleSection { heading: "Pack intentionally", body: "Bring easy rewards, cleanup supplies, water, and any class materials requested by the business." },
            ArticleSection { heading: "Ask health questions early", body: "For vaccination, illness, or medical concerns, consult a qualified veterinarian before class." },
        ],
    },
    ResourceArticle {
        slug: "basic-leash-skills-to-practice-at-home",
        title: "Basic Leash Skills to Practice at Home",
        description: "Simple leash-skill ideas dog owners can practice at home before asking about training support.",
        service_slug: "dog-training",
        published: "2026-07-19",
        modified: "2026-07-19",
        sections: &[
            ArticleSection { heading: "Reward attention", body: "Practice rewarding your dog for checking in before adding distractions or longer walks." },
            ArticleSection { heading: "Keep sessions short", body: "Short, successful practice can be more useful than long sessions where the dog becomes tired or frustrated." },
            ArticleSection { heading: "Use safe equipment", body: "Choose equipment that fits properly and ask a professional or veterinarian about safety concerns." },
        ],
    },
    ResourceArticle {
        slug: "how-to-choose-a-dog-trainer-in-lorain-county",
        title: "How to Choose a Dog Trainer in Lorain County",
        description: "Questions Lorain County dog owners can ask when evaluating a trainer, class, or training program.",
        service_slug: "dog-training",
        published: "2026-07-19",
        modified: "2026-07-19",
        sections: &[
            ArticleSection { heading: "Ask about fit", body: "Describe your dog honestly and ask whether the format matches your dog's needs and safety profile." },
            ArticleSection { heading: "Look for clarity", body: "A useful training conversation should explain process, expectations, preparation, and how questions are handled." },
            ArticleSection { heading: "Confirm policies", body: "Before booking, confirm credentials, insurance, cancellation rules, health requirements, and pricing directly with the business." },
        ],
    },
    ResourceArticle {
        slug: "questions-to-ask-before-joining-a-group-dog-class",
        title: "Questions to Ask Before Joining a Group Dog Class",
        description: "Use these practical questions to decide whether a group dog class is a safe and useful fit.",
        service_slug: "group-dog-classes",
        published: "2026-07-19",
        modified: "2026-07-19",
        sections: &[
            ArticleSection { heading: "Class structure", body: "Ask how long classes run, how many dogs may attend, and how dogs are introduced to new exercises." },
            ArticleSection { heading: "Dog readiness", body: "Share barking, lunging, fear, overexcitement, or handling concerns before arriving." },
            ArticleSection { heading: "Owner expectations", body: "Ask what to practice at home and what progress is realistic for the class length." },
        ],
    },
    ResourceArticle {
        slug: "puppy-socialization-without-overwhelming-your-puppy",
        title: "Puppy Socialization Without Overwhelming Your Puppy",
        description: "A calm approach to puppy socialization that prioritizes confidence, safety, and veterinarian guidance.",
        service_slug: "puppy-training",
        published: "2026-07-19",
        modified: "2026-07-19",
        sections: &[
            ArticleSection { heading: "Think exposure, not pressure", body: "Let your puppy notice new sights and sounds from a comfortable distance." },
            ArticleSection { heading: "Watch body language", body: "Pausing, hiding, frantic pulling, or refusing food can mean the setup is too difficult." },
            ArticleSection { heading: "Protect health", body: "Ask a qualified veterinarian about safe public exposure before your puppy is fully protected." },
        ],
    },
    ResourceArticle {
        slug: "helping-a-dog-stay-focused-around-distractions",
        title: "Helping a Dog Stay Focused Around Distractions",
        description: "Practical ways to build focus around everyday distractions without expecting instant results.",
        service_slug: "dog-training",
        published: "2026-07-19",
        modified: "2026-07-19",
        sections: &[
            ArticleSection { heading: "Lower the difficulty", body: "Start far enough from distractions that your dog can still respond and take rewards." },
            ArticleSection { heading: "Practice one skill", body: "Choose one simple cue or behavior and reward it consistently before adding complexity." },
            ArticleSection { heading: "Track patterns", body: "Note where focus improves or breaks down so your inquiry includes useful context." },
        ],
    },
    ResourceArticle {
        slug: "what-to-bring-to-a-dog-training-class",
        title: "What to Bring to a Dog Training Class",
        description: "A simple packing list for dog training or puppy class, plus questions to confirm before attending.",
        service_slug: "group-dog-classes",
        published: "2026-07-19",
        modified: "2026-07-19",
        sections: &[
            ArticleSection { heading: "Core supplies", body: "Bring a leash, properly fitted collar or harness, rewards your dog can eat easily, water, and cleanup supplies." },
            ArticleSection { heading: "Helpful notes", body: "Write down current goals, questions, medications or health limits, and any behavior concerns." },
            ArticleSection { heading: "Confirm requirements", body: "Ask Cooper & Co. about class-specific requirements before arriving." },
        ],
    },
    ResourceArticle {
        slug: "dog-training-goals-how-to-set-realistic-expectations",
        title: "Dog Training Goals: How to Set Realistic Expectations",
        description: "Set practical dog training goals that account for practice, environment, dog age, and safety.",
        service_slug: "dog-training",
        published: "2026-07-19",
        modified: "2026-07-19",
        sections: &[
            ArticleSection { heading: "Define one priority", body: "Choose the skill or routine that would make the biggest practical difference first." },
            ArticleSection { heading: "Measure small wins", body: "Look for shorter recovery time, calmer starts, or better focus, not perfection after one session." },
            ArticleSection { heading: "Adjust as needed", body: "Training plans should change when health, safety, or environment creates new information." },
        ],
    },
    ResourceArticle {
        slug: "indoor-dog-enrichment-ideas-for-ohio-winters",
        title: "Indoor Dog-Enrichment Ideas for Ohio Winters",
        description: "Low-pressure indoor enrichment ideas for cold Ohio weather when outdoor practice is limited.",
        service_slug: "dog-training",
        published: "2026-07-19",
        modified: "2026-07-19",
        sections: &[
            ArticleSection { heading: "Use food puzzles", body: "Scatter feeding, safe puzzle toys, and short nose-work games can give dogs a useful outlet indoors." },
            ArticleSection { heading: "Practice calm skills", body: "Winter days can be a good time for short place, settle, leash, or recall sessions indoors." },
            ArticleSection { heading: "Watch health limits", body: "Ask a veterinarian about exercise restrictions, weight, pain, or medical concerns." },
        ],
    },
];

#[derive(Debug)]
pub enum MarketingResponse {
    Html {
        status: Status,
        body: String,
        x_robots: bool,
    },
    Xml(String),
    Text(String),
    Redirect(Redirect),
    File {
        file: NamedFile,
        x_robots: bool,
    },
}

impl<'r> Responder<'r, 'static> for MarketingResponse {
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        match self {
            MarketingResponse::Html {
                status,
                body,
                x_robots,
            } => {
                let mut response = Response::build();
                response.status(status);
                response.header(ContentType::HTML);
                response.raw_header("Cache-Control", "public, max-age=300");
                if x_robots {
                    response.header(Header::new("X-Robots-Tag", "noindex, nofollow"));
                }
                response.sized_body(body.len(), std::io::Cursor::new(body));
                response.ok()
            }
            MarketingResponse::Xml(body) => RawXml(body).respond_to(request),
            MarketingResponse::Text(body) => {
                let mut response = Response::build();
                response.status(Status::Ok);
                response.header(ContentType::Plain);
                response.sized_body(body.len(), std::io::Cursor::new(body));
                response.ok()
            }
            MarketingResponse::Redirect(redirect) => redirect.respond_to(request),
            MarketingResponse::File { file, x_robots } => {
                let response = file.respond_to(request)?;
                let mut builder = Response::build_from(response);
                builder.raw_header("Cache-Control", "public, max-age=31536000, immutable");
                if x_robots {
                    builder.header(Header::new("X-Robots-Tag", "noindex, nofollow"));
                }
                builder.ok()
            }
        }
    }
}

#[get("/")]
pub async fn home_page() -> MarketingResponse {
    render_marketing_path("/").await
}

#[get("/<path..>", rank = 20)]
pub async fn marketing_page(path: PathBuf) -> MarketingResponse {
    let path = format!("/{}", path.to_string_lossy().replace('\\', "/"));
    render_marketing_path(&path).await
}

#[get("/robots.txt")]
pub fn robots_txt() -> MarketingResponse {
    MarketingResponse::Text(robots_body())
}

#[get("/robots")]
pub fn robots() -> MarketingResponse {
    robots_txt()
}

#[get("/sitemap.xml")]
pub fn sitemap_xml() -> MarketingResponse {
    MarketingResponse::Xml(sitemap_body())
}

async fn render_marketing_path(path: &str) -> MarketingResponse {
    if let Some(file_response) = static_file_response(path).await {
        return file_response;
    }

    let normalized = normalize_path(path);
    if normalized != path && path != "/" {
        return MarketingResponse::Redirect(Redirect::permanent(normalized));
    }

    if normalized == "/admin" || normalized.starts_with("/admin/") {
        return MarketingResponse::Html {
            status: Status::Ok,
            body: admin_shell_html(),
            x_robots: true,
        };
    }

    if let Some(target) = obsolete_redirect(&normalized) {
        return MarketingResponse::Redirect(Redirect::permanent(target));
    }

    if is_gone_path(&normalized) {
        return MarketingResponse::Html {
            status: Status::Gone,
            body: error_page(
                Status::Gone,
                "This Cooper & Co. page has been retired",
                "Use the current Lorain-based service and contact pages instead.",
            ),
            x_robots: true,
        };
    }

    match page_for_path(&normalized) {
        Some(page) => MarketingResponse::Html {
            status: Status::Ok,
            body: render_page(&page),
            x_robots: staging_noindex_enabled() || !page.indexable,
        },
        None => MarketingResponse::Html {
            status: Status::NotFound,
            body: error_page(
                Status::NotFound,
                "This Cooper & Co. page was not found",
                "Use the current service, resource, or contact links to continue.",
            ),
            x_robots: true,
        },
    }
}

async fn static_file_response(path: &str) -> Option<MarketingResponse> {
    if path == "/styles.css" {
        let candidates = [
            PathBuf::from("frontend/styles.css"),
            PathBuf::from("../frontend/styles.css"),
        ];
        for candidate in candidates {
            if let Ok(file) = NamedFile::open(candidate).await {
                return Some(MarketingResponse::File {
                    file,
                    x_robots: staging_noindex_enabled(),
                });
            }
        }
    }

    if !path
        .rsplit('/')
        .next()
        .is_some_and(|segment| segment.contains('.'))
    {
        return None;
    }

    let relative = path.trim_start_matches('/');
    let candidate = static_dir().join(relative);
    NamedFile::open(candidate)
        .await
        .ok()
        .map(|file| MarketingResponse::File {
            file,
            x_robots: staging_noindex_enabled(),
        })
}

pub fn robots_body() -> String {
    if staging_noindex_enabled() {
        "User-agent: *\nDisallow: /\n".to_owned()
    } else {
        format!(
            "User-agent: *\nAllow: /\nDisallow: /admin\nDisallow: /api/\nDisallow: /auth/\nSitemap: {}/sitemap.xml\n",
            canonical_origin()
        )
    }
}

pub fn sitemap_body() -> String {
    let urls = indexable_paths()
        .iter()
        .map(|path| {
            format!(
                "<url><loc>{}{}</loc><lastmod>{LASTMOD}</lastmod><changefreq>monthly</changefreq><priority>{}</priority></url>",
                canonical_origin(),
                path,
                if path == "/" { "1.0" } else { "0.8" }
            )
        })
        .collect::<String>();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">{urls}</urlset>"#
    )
}

pub fn indexable_paths() -> Vec<String> {
    let mut paths = [
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
        "/accessibility",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();

    paths.extend(
        confirmed_service_areas()
            .iter()
            .map(|area| format!("/service-areas/{}", area.slug)),
    );
    paths.extend(
        ARTICLES
            .iter()
            .map(|article| format!("/resources/{}", article.slug)),
    );
    paths
}

pub fn page_for_path(path: &str) -> Option<Page> {
    match path {
        "/" => Some(home()),
        "/about" => Some(about()),
        "/services" => Some(services_index()),
        "/contact" => Some(contact()),
        "/faq" => Some(faq_page()),
        "/service-areas" => Some(service_areas_index()),
        "/resources" => Some(resources_index()),
        "/privacy" => Some(privacy()),
        "/accessibility" => Some(accessibility()),
        _ => {
            if let Some(slug) = path.strip_prefix("/services/") {
                return SERVICES
                    .iter()
                    .find(|service| service.slug == slug)
                    .map(service_page);
            }
            if let Some(slug) = path.strip_prefix("/service-areas/") {
                return confirmed_service_areas()
                    .iter()
                    .find(|area| area.slug == slug)
                    .map(|area| location_page(area));
            }
            if let Some(slug) = path.strip_prefix("/resources/") {
                return ARTICLES
                    .iter()
                    .find(|article| article.slug == slug)
                    .map(article_page);
            }
            None
        }
    }
}

#[cfg(test)]
pub fn validation_errors() -> Vec<String> {
    let mut errors = Vec::new();
    let mut canonicals = std::collections::HashSet::new();

    for path in indexable_paths() {
        let Some(page) = page_for_path(&path) else {
            errors.push(format!("sitemap URL lacks a route: {path}"));
            continue;
        };
        if page.title.trim().is_empty() {
            errors.push(format!("missing title: {path}"));
        }
        if page.description.trim().is_empty() {
            errors.push(format!("missing description: {path}"));
        }
        if page.h1.trim().is_empty() {
            errors.push(format!("missing H1: {path}"));
        }
        if !canonicals.insert(page.path.clone()) {
            errors.push(format!("duplicate canonical path: {}", page.path));
        }
        if path.starts_with("/service-areas/")
            && confirmed_service_areas()
                .iter()
                .all(|area| format!("/service-areas/{}", area.slug) != path)
        {
            errors.push(format!("indexable location is unconfirmed: {path}"));
        }
        let rendered = render_page(&page);
        for blocked in ["TODO", "placeholder.example", "example.com"] {
            if rendered.contains(blocked) {
                errors.push(format!("production-facing {blocked} remains on {path}"));
            }
        }
        for schema in json_ld_blocks(&rendered) {
            if let Err(error) = serde_json::from_str::<Value>(&schema) {
                errors.push(format!("invalid JSON-LD on {path}: {error}"));
            }
        }
    }

    errors
}

#[cfg(test)]
pub fn json_ld_blocks(html: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut rest = html;
    let marker = r#"<script type="application/ld+json">"#;
    while let Some(start) = rest.find(marker) {
        let after_start = &rest[start + marker.len()..];
        let Some(end) = after_start.find("</script>") else {
            break;
        };
        blocks.push(after_start[..end].trim().to_owned());
        rest = &after_start[end + "</script>".len()..];
    }
    blocks
}

fn home() -> Page {
    let services = SERVICES
        .iter()
        .map(|service| {
            format!(
                r#"<article class="card"><h3><a href="/services/{slug}">{name}</a></h3><p>{summary}</p></article>"#,
                slug = service.slug,
                name = escape(service.name),
                summary = escape(service.summary)
            )
        })
        .collect::<String>();
    let resources = ARTICLES
        .iter()
        .take(3)
        .map(resource_card)
        .collect::<String>();

    let body = format!(
        r#"
<section class="hero" aria-labelledby="home-title">
  <picture class="hero-media">
    <source srcset="/assets/cooperco-pet-services-hero.avif" type="image/avif">
    <img class="hero-image" src="/assets/cooperco-pet-services-hero.webp" alt="{image_alt}" width="1600" height="900" fetchpriority="high" decoding="async">
  </picture>
  <div class="hero-copy">
    <p class="eyebrow">Pet service based in Lorain, Ohio</p>
    <h1 id="home-title">Cooper &amp; Co. dog training and pet services in Lorain, Ohio</h1>
    <p>Cooper &amp; Co. is based in Lorain and serves pet owners across Lorain County and nearby communities. Ask about dog training, puppy training, and group dog classes.</p>
    <div class="hero-actions"><a class="button primary" href="/contact">Request information</a><a class="button secondary" href="tel:{phone_e164}">{phone}</a></div>
  </div>
</section>
<section class="section" aria-labelledby="home-services"><div class="section-heading"><p class="eyebrow">Services</p><h2 id="home-services">Confirmed service pages</h2><p>These pages cover services supported by current repository evidence without adding unconfirmed claims.</p></div><div class="service-grid">{services}</div></section>
<section class="section split" aria-labelledby="classes-overview"><div><p class="eyebrow">Classes</p><h2 id="classes-overview">Group and puppy training inquiries</h2><p>Class times, openings, requirements, and pricing should be confirmed directly with Cooper &amp; Co. before scheduling.</p></div><article class="update"><span>Current next step</span><h3>Share your goals before booking</h3><p>Use the inquiry form to describe your dog, location, goals, and preferred timeframe.</p><a href="/contact">Contact Cooper &amp; Co.</a></article></section>
<section class="section" aria-labelledby="area-overview"><div class="section-heading"><p class="eyebrow">Service Area</p><h2 id="area-overview">Based in Lorain and focused on Lorain County</h2><p>The site only publishes indexable city pages when the owner marks a location confirmed.</p></div><div class="service-grid"><article class="card"><h3><a href="/service-areas/lorain-oh">Lorain, OH</a></h3><p>Lorain is the confirmed home-city service-area page.</p></article><article class="card"><h3><a href="/service-areas">Nearby communities</a></h3><p>Candidate nearby communities are documented for owner review before publication.</p></article><article class="card"><h3><a href="/contact">Confirm availability</a></h3><p>Contact Cooper &amp; Co. with your city or ZIP code and service goals.</p></article></div></section>
<section class="section" aria-labelledby="process"><div class="section-heading"><p class="eyebrow">Process</p><h2 id="process">How inquiries work</h2></div><div class="service-grid"><article class="card"><h3>1. Send details</h3><p>Provide your contact details, city or ZIP code, pet age, service interest, and goals.</p></article><article class="card"><h3>2. Confirm fit</h3><p>Cooper &amp; Co. can confirm availability, class fit, and any requirements directly.</p></article><article class="card"><h3>3. Plan next steps</h3><p>You receive the appropriate scheduling or follow-up path from the business.</p></article></div></section>
<section class="section trust-section" aria-labelledby="trust"><div class="section-heading"><p class="eyebrow">Trust</p><h2 id="trust">Verified public facts only</h2><p>The website uses the confirmed business name, Lorain location, phone number, email, and public social links. Testimonials, credentials, insurance, hours, and policies are reserved for owner-approved content.</p></div></section>
<section class="section faq" aria-labelledby="home-faq"><div class="section-heading"><p class="eyebrow">FAQ</p><h2 id="home-faq">Common questions</h2></div>{faq}</section>
<section class="section" aria-labelledby="resource-preview"><div class="section-heading"><p class="eyebrow">Resources</p><h2 id="resource-preview">Helpful dog training articles</h2></div><div class="service-grid">{resources}</div></section>
{contact_section}
"#,
        image_alt = SOCIAL_IMAGE_ALT,
        phone_e164 = BUSINESS.phone_e164,
        phone = BUSINESS.phone,
        services = services,
        resources = resources,
        faq = faq_markup(&[
            FaqItem { question: "Where is Cooper & Co. based?", answer: "Cooper & Co. is based in Lorain, Ohio." },
            FaqItem { question: "Which services are published on the website?", answer: "The published service pages are dog training, puppy training, and group dog classes." },
            FaqItem { question: "Are nearby communities served?", answer: "The homepage uses cautious Lorain County wording. Individual location pages are only published after owner confirmation." },
        ]),
        contact_section = contact_section("Ask about dog training or classes"),
    );

    Page {
        path: "/".to_owned(),
        title: "Cooper & Co. | Dog Training in Lorain, Ohio".to_owned(),
        description: "Cooper & Co. is based in Lorain, Ohio. Ask about dog training, puppy training, and group dog classes for Lorain County pet owners.".to_owned(),
        h1: "Cooper & Co. dog training and pet services in Lorain, Ohio".to_owned(),
        body,
        breadcrumbs: vec![("Home", "/".to_owned())],
        schema: vec![organization_schema(), website_schema(), webpage_schema("/", "WebPage")],
        indexable: true,
    }
}

fn about() -> Page {
    basic_page(
        "/about",
        "About Cooper & Co. in Lorain, Ohio | Cooper & Co.",
        "Learn about Cooper & Co., a Lorain, Ohio pet-service business with dog training and class inquiry pages.",
        "About Cooper & Co.",
        "Cooper & Co. is a Lorain, Ohio pet-service business. The website is intentionally careful: it publishes confirmed contact and location information while reserving credentials, hours, pricing, testimonials, and policy details for owner approval.",
        "AboutPage",
    )
}

fn services_index() -> Page {
    let cards = SERVICES.iter().map(service_card).collect::<String>();
    let body = format!(
        r#"<section class="section page-hero" aria-labelledby="services-title"><p class="eyebrow">Services</p><h1 id="services-title">Dog training services from Cooper &amp; Co.</h1><p>Use these pages to understand service fit and what to include when contacting Cooper &amp; Co.</p></section><section class="section"><div class="service-grid">{cards}</div></section>{contact}"#,
        contact = contact_section("Ask which training option fits your dog")
    );
    Page {
        path: "/services".to_owned(),
        title: "Dog Training Services in Lorain, OH | Cooper & Co.".to_owned(),
        description: "Explore Cooper & Co. dog training, puppy training, and group class inquiry pages for Lorain-area pet owners.".to_owned(),
        h1: "Dog training services from Cooper & Co.".to_owned(),
        body,
        breadcrumbs: vec![("Home", "/".to_owned()), ("Services", "/services".to_owned())],
        schema: vec![webpage_schema("/services", "WebPage"), service_collection_schema()],
        indexable: true,
    }
}

fn service_page(service: &ServiceDefinition) -> Page {
    let related = service
        .related_resources
        .iter()
        .filter_map(|slug| ARTICLES.iter().find(|article| article.slug == *slug))
        .map(resource_card)
        .collect::<String>();
    let body = format!(
        r#"<section class="section page-hero" aria-labelledby="service-title"><p class="eyebrow">Service</p><h1 id="service-title">{h1}</h1><p>{summary}</p><div class="hero-actions"><a class="button primary" href="/contact">Request information</a><a class="button secondary on-light" href="tel:{phone_e164}">{phone}</a></div></section>
<section class="section" aria-labelledby="service-fit"><div class="section-heading"><p class="eyebrow">Fit</p><h2 id="service-fit">Who this may help</h2><p>{audience}</p></div></section>
<section class="section split" aria-labelledby="service-process"><div><p class="eyebrow">Process</p><h2 id="service-process">Expected inquiry process</h2>{process}</div><div><p class="eyebrow">Prepare</p><h2>What to share</h2>{prepare}</div></section>
<section class="section" aria-labelledby="availability"><div class="section-heading"><p class="eyebrow">Availability</p><h2 id="availability">Geographic availability</h2><p>Cooper &amp; Co. is based in Lorain, Ohio. For other Lorain County communities, submit your city or ZIP code so the business can confirm fit.</p></div><a class="button secondary on-light" href="/service-areas">View service-area status</a></section>
<section class="section faq" aria-labelledby="service-faq"><div class="section-heading"><p class="eyebrow">FAQ</p><h2 id="service-faq">Service questions</h2></div>{faq}</section>
<section class="section" aria-labelledby="related"><div class="section-heading"><p class="eyebrow">Resources</p><h2 id="related">Related resources</h2></div><div class="service-grid">{related}</div></section>
{contact}"#,
        h1 = escape(service.name),
        summary = escape(service.summary),
        audience = escape(service.audience),
        process = list_markup(service.process),
        prepare = list_markup(service.prepare),
        faq = faq_markup(service.faq),
        related = related,
        phone_e164 = BUSINESS.phone_e164,
        phone = BUSINESS.phone,
        contact = contact_section("Ask about this service"),
    );
    let path = format!("/services/{}", service.slug);
    Page {
        path: path.clone(),
        title: service.page_title.to_owned(),
        description: service.description.to_owned(),
        h1: service.name.to_owned(),
        body,
        breadcrumbs: vec![
            ("Home", "/".to_owned()),
            ("Services", "/services".to_owned()),
            (service.name, path.clone()),
        ],
        schema: vec![webpage_schema(&path, "WebPage"), service_schema(service)],
        indexable: true,
    }
}

fn service_areas_index() -> Page {
    let confirmed = confirmed_service_areas()
        .iter()
        .map(|area| {
            format!(
                r#"<article class="card"><h3><a href="/service-areas/{slug}">{name}</a></h3><p>Confirmed public service-area page.</p></article>"#,
                slug = area.slug,
                name = escape(area.name)
            )
        })
        .collect::<String>();
    let candidates = SERVICE_AREAS
        .iter()
        .filter(|area| area.status == LocationStatus::Unconfirmed)
        .map(|area| format!("<li>{}</li>", escape(area.name)))
        .collect::<String>();
    let body = format!(
        r#"<section class="section page-hero" aria-labelledby="areas-title"><p class="eyebrow">Service Areas</p><h1 id="areas-title">Cooper &amp; Co. service areas</h1><p>Cooper &amp; Co. is based in Lorain, Ohio. Additional Lorain County communities are candidates until the owner confirms active coverage.</p></section><section class="section"><div class="section-heading"><h2>Confirmed location pages</h2></div><div class="service-grid">{confirmed}</div></section><section class="section"><div class="section-heading"><h2>Candidate nearby communities</h2><p>These communities are stored as unconfirmed and are not included in the sitemap as location pages.</p></div><ul>{candidates}</ul></section>{contact}"#,
        contact = contact_section("Confirm service availability in your city")
    );
    Page {
        path: "/service-areas".to_owned(),
        title: "Service Areas in Lorain County | Cooper & Co.".to_owned(),
        description: "See Cooper & Co. service-area status for Lorain, Ohio and nearby Lorain County communities requiring owner confirmation.".to_owned(),
        h1: "Cooper & Co. service areas".to_owned(),
        body,
        breadcrumbs: vec![
            ("Home", "/".to_owned()),
            ("Service Areas", "/service-areas".to_owned()),
        ],
        schema: vec![webpage_schema("/service-areas", "WebPage")],
        indexable: true,
    }
}

fn location_page(area: &ServiceArea) -> Page {
    let services = SERVICES.iter().map(service_card).collect::<String>();
    let body = format!(
        r#"<section class="section page-hero" aria-labelledby="location-title"><p class="eyebrow">Confirmed Location</p><h1 id="location-title">Dog training in {name}</h1><p>Cooper &amp; Co. is based in {home_city}, {state}. This page is published because {name} is the confirmed home-city service area.</p><div class="hero-actions"><a class="button primary" href="/contact">Request information</a><a class="button secondary on-light" href="/services">View services</a></div></section><section class="section" aria-labelledby="location-services"><div class="section-heading"><p class="eyebrow">Services</p><h2 id="location-services">Services available for inquiry</h2><p>Ask about scheduling, class fit, and requirements before booking.</p></div><div class="service-grid">{services}</div></section><section class="section faq" aria-labelledby="location-faq"><div class="section-heading"><p class="eyebrow">FAQ</p><h2 id="location-faq">Local questions</h2></div>{faq}</section>{contact}"#,
        name = escape(area.name),
        home_city = BUSINESS.home_city,
        state = BUSINESS.state,
        services = services,
        faq = faq_markup(&[
            FaqItem { question: "Is there a public office address?", answer: "No public street address is published until the owner confirms a customer-facing address policy." },
            FaqItem { question: "Can I ask about nearby communities?", answer: "Yes. Include your city or ZIP code in the inquiry so Cooper & Co. can confirm current availability." },
        ]),
        contact = contact_section("Ask about local scheduling"),
    );
    let path = format!("/service-areas/{}", area.slug);
    Page {
        path: path.clone(),
        title: format!("Dog Training in {} | Cooper & Co.", area.name),
        description: format!(
            "Ask Cooper & Co. about dog training, puppy training, and group dog classes in {}.",
            area.name
        ),
        h1: format!("Dog training in {}", area.name),
        body,
        breadcrumbs: vec![
            ("Home", "/".to_owned()),
            ("Service Areas", "/service-areas".to_owned()),
            (area.name, path.clone()),
        ],
        schema: vec![webpage_schema(&path, "WebPage"), local_business_schema()],
        indexable: true,
    }
}

fn resources_index() -> Page {
    let cards = ARTICLES.iter().map(resource_card).collect::<String>();
    let body = format!(
        r#"<section class="section page-hero" aria-labelledby="resources-title"><p class="eyebrow">Resources</p><h1 id="resources-title">Dog training resources</h1><p>Educational articles help owners prepare thoughtful questions before contacting Cooper &amp; Co. Medical concerns should be directed to a qualified veterinarian.</p></section><section class="section"><div class="service-grid">{cards}</div></section>{contact}"#,
        contact = contact_section("Ask a dog training question")
    );
    Page {
        path: "/resources".to_owned(),
        title: "Dog Training Resources | Cooper & Co.".to_owned(),
        description: "Read Cooper & Co. resources about group classes, puppy preparation, leash skills, and training expectations.".to_owned(),
        h1: "Dog training resources".to_owned(),
        body,
        breadcrumbs: vec![("Home", "/".to_owned()), ("Resources", "/resources".to_owned())],
        schema: vec![webpage_schema("/resources", "WebPage")],
        indexable: true,
    }
}

fn article_page(article: &ResourceArticle) -> Page {
    let sections = article
        .sections
        .iter()
        .map(|section| {
            format!(
                r#"<h2>{}</h2><p>{}</p>"#,
                escape(section.heading),
                escape(section.body)
            )
        })
        .collect::<String>();
    let related_service = SERVICES
        .iter()
        .find(|service| service.slug == article.service_slug)
        .unwrap_or(&SERVICES[0]);
    let related = ARTICLES
        .iter()
        .filter(|item| item.slug != article.slug && item.service_slug == article.service_slug)
        .take(2)
        .map(resource_card)
        .collect::<String>();
    let body = format!(
        r#"<article class="section page-hero resource-article" aria-labelledby="article-title"><p class="eyebrow">Cooper &amp; Co. Resource</p><h1 id="article-title">{title}</h1><p>{description}</p><p><strong>By Cooper &amp; Co.</strong> Published <time datetime="{published}">{published}</time>; updated <time datetime="{modified}">{modified}</time>.</p><div class="article-body">{sections}<h2>When to ask for help</h2><p>Contact Cooper &amp; Co. with your dog details, goals, and location. For medical concerns, consult a qualified veterinarian.</p></div><div class="hero-actions"><a class="button primary" href="/contact">Contact Cooper &amp; Co.</a><a class="button secondary on-light" href="/services/{service_slug}">{service_name}</a></div></article><section class="section" aria-labelledby="related-articles"><div class="section-heading"><p class="eyebrow">Related</p><h2 id="related-articles">Related articles</h2></div><div class="service-grid">{related}</div></section>"#,
        title = escape(article.title),
        description = escape(article.description),
        published = article.published,
        modified = article.modified,
        sections = sections,
        service_slug = related_service.slug,
        service_name = escape(related_service.name),
        related = related,
    );
    let path = format!("/resources/{}", article.slug);
    Page {
        path: path.clone(),
        title: format!("{} | Cooper & Co.", article.title),
        description: article.description.to_owned(),
        h1: article.title.to_owned(),
        body,
        breadcrumbs: vec![
            ("Home", "/".to_owned()),
            ("Resources", "/resources".to_owned()),
            (article.title, path.clone()),
        ],
        schema: vec![webpage_schema(&path, "WebPage"), article_schema(article)],
        indexable: true,
    }
}

fn contact() -> Page {
    let body = format!(
        r#"<section class="section page-hero" aria-labelledby="contact-title"><p class="eyebrow">Contact</p><h1 id="contact-title">Contact Cooper &amp; Co.</h1><p>Send a careful inquiry with only the details needed to respond about dog training, puppy training, or group dog classes.</p></section>{contact}"#,
        contact = contact_form()
    );
    Page {
        path: "/contact".to_owned(),
        title: "Contact Cooper & Co. in Lorain, Ohio".to_owned(),
        description: "Contact Cooper & Co. by phone, email, Facebook, or inquiry form about Lorain-area dog training and classes.".to_owned(),
        h1: "Contact Cooper & Co.".to_owned(),
        body,
        breadcrumbs: vec![("Home", "/".to_owned()), ("Contact", "/contact".to_owned())],
        schema: vec![webpage_schema("/contact", "ContactPage")],
        indexable: true,
    }
}

fn faq_page() -> Page {
    let body = format!(
        r#"<section class="section page-hero" aria-labelledby="faq-title"><p class="eyebrow">FAQ</p><h1 id="faq-title">Cooper &amp; Co. questions</h1><p>Answers are limited to confirmed website information. Ask the owner directly for current prices, requirements, hours, and availability.</p></section><section class="section faq">{faq}</section>{contact}"#,
        faq = faq_markup(&[
            FaqItem { question: "What services are listed?", answer: "Dog training, puppy training, and group dog classes are the confirmed service pages." },
            FaqItem { question: "Where is Cooper & Co. based?", answer: "Cooper & Co. is based in Lorain, Ohio." },
            FaqItem { question: "Are hours or prices published?", answer: "Hours and prices require owner confirmation and are not published as fixed claims." },
            FaqItem { question: "What should I send?", answer: "Send your contact details, city or ZIP code, pet name and age, service interest, goals, and preferred timeframe." },
        ]),
        contact = contact_section("Still have a question?"),
    );
    Page {
        path: "/faq".to_owned(),
        title: "FAQ | Cooper & Co. Dog Training in Lorain".to_owned(),
        description: "Find answers about Cooper & Co. dog training inquiries, service-area confirmation, pricing policy, and contact details.".to_owned(),
        h1: "Cooper & Co. questions".to_owned(),
        body,
        breadcrumbs: vec![("Home", "/".to_owned()), ("FAQ", "/faq".to_owned())],
        schema: vec![webpage_schema("/faq", "FAQPage")],
        indexable: true,
    }
}

fn privacy() -> Page {
    basic_page(
        "/privacy",
        "Privacy Policy | Cooper & Co.",
        "Read how Cooper & Co. website inquiries collect contact and pet-service details for follow-up.",
        "Privacy policy",
        "The inquiry form collects contact information and pet-service details so Cooper & Co. can respond. Do not submit private medical details, financial information, or emergency information through the website.",
        "WebPage",
    )
}

fn accessibility() -> Page {
    basic_page(
        "/accessibility",
        "Accessibility Statement | Cooper & Co.",
        "Accessibility statement for the Cooper & Co. website, built for semantic navigation, readable content, and keyboard access.",
        "Accessibility statement",
        "The site aims to provide semantic headings, keyboard-friendly navigation, visible focus states, meaningful link text, and readable forms. Accessibility feedback can be sent through the contact options on this site.",
        "WebPage",
    )
}

fn basic_page(
    path: &'static str,
    title: &'static str,
    description: &'static str,
    h1: &'static str,
    copy: &'static str,
    schema_type: &'static str,
) -> Page {
    Page {
        path: path.to_owned(),
        title: title.to_owned(),
        description: description.to_owned(),
        h1: h1.to_owned(),
        body: format!(
            r#"<section class="section page-hero" aria-labelledby="basic-title"><p class="eyebrow">Cooper &amp; Co.</p><h1 id="basic-title">{}</h1><p>{}</p></section>{}"#,
            escape(h1),
            escape(copy),
            contact_section("Contact Cooper & Co.")
        ),
        breadcrumbs: vec![("Home", "/".to_owned()), (h1, path.to_owned())],
        schema: vec![webpage_schema(path, schema_type)],
        indexable: true,
    }
}

fn render_page(page: &Page) -> String {
    let _validated_h1 = &page.h1;
    let canonical = format!("{}{}", canonical_origin(), page.path);
    let robots = if staging_noindex_enabled() || !page.indexable {
        "noindex, nofollow"
    } else {
        "index, follow, max-image-preview:large"
    };
    let schema = json!({
        "@context": "https://schema.org",
        "@graph": page.schema
    });
    let hooks = verification_and_analytics_hooks();
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="{robots}">
<title>{title}</title>
<meta name="description" content="{description}">
<link rel="canonical" href="{canonical}">
<link rel="icon" type="image/png" href="/assets/favicon.png">
<link rel="stylesheet" href="/styles.css">
<meta name="theme-color" content="#285c4d">
<meta property="og:type" content="website">
<meta property="og:locale" content="en_US">
<meta property="og:site_name" content="{site_name}">
<meta property="og:title" content="{title}">
<meta property="og:description" content="{description}">
<meta property="og:url" content="{canonical}">
<meta property="og:image" content="{origin}{social_image}">
<meta property="og:image:alt" content="{social_alt}">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="{title}">
<meta name="twitter:description" content="{description}">
<meta name="twitter:image" content="{origin}{social_image}">
<meta name="twitter:image:alt" content="{social_alt}">
{hooks}
<script type="application/ld+json">{schema}</script>
</head>
<body>
<a class="skip-link" href="#content">Skip to content</a>
{header}
<main id="content">
{breadcrumbs}
{body}
</main>
{footer}
</body>
</html>"##,
        robots = robots,
        title = escape_attr(&page.title),
        description = escape_attr(&page.description),
        canonical = escape_attr(&canonical),
        site_name = escape_attr(BUSINESS.name),
        origin = canonical_origin(),
        social_image = SOCIAL_IMAGE,
        social_alt = escape_attr(SOCIAL_IMAGE_ALT),
        hooks = hooks,
        schema = schema,
        header = header(),
        breadcrumbs = breadcrumbs(&page.breadcrumbs),
        body = page.body,
        footer = footer(),
    )
}

fn error_page(status: Status, h1: &str, copy: &str) -> String {
    let page = Page {
        path: format!("/{}", status.code),
        title: format!("{h1} | Cooper & Co."),
        description: copy.to_owned(),
        h1: h1.to_owned(),
        body: format!(
            r#"<section class="section page-hero" aria-labelledby="error-title"><p class="eyebrow">{}</p><h1 id="error-title">{}</h1><p>{}</p><div class="hero-actions"><a class="button primary" href="/services">View services</a><a class="button secondary on-light" href="/contact">Contact</a></div></section>"#,
            status.code,
            escape(h1),
            escape(copy)
        ),
        breadcrumbs: vec![("Home", "/".to_owned())],
        schema: vec![webpage_schema("/", "WebPage")],
        indexable: false,
    };
    render_page(&page)
}

fn admin_shell_html() -> String {
    r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><meta name="robots" content="noindex, nofollow"><title>Admin | Cooper &amp; Co.</title><link rel="stylesheet" href="/styles.css"></head><body><main id="app" class="admin-shell"><section class="admin-panel"><h1>Cooper &amp; Co. admin</h1><p>Sign in is required to manage inquiries.</p><a class="button primary" href="/auth/microsoft/login">Sign in with Microsoft</a></section></main></body></html>"#.to_owned()
}

fn header() -> String {
    r#"<header class="topbar"><a class="brand" href="/" aria-label="Cooper and Co home"><span class="brand-mark" aria-hidden="true">C&amp;Co</span><span>Cooper &amp; Co.</span></a><nav aria-label="Main navigation"><a href="/about">About</a><a href="/services">Services</a><a href="/service-areas">Service areas</a><a href="/resources">Resources</a><a href="/faq">FAQ</a><a href="/contact">Contact</a></nav></header>"#.to_owned()
}

fn footer() -> String {
    format!(
        r#"<footer><span>Cooper &amp; Co. · {city}, {state} · <a href="tel:{phone_e164}">{phone}</a></span><a href="/privacy">Privacy</a><a href="/accessibility">Accessibility</a><a href="{facebook}" rel="noreferrer">Facebook</a></footer>"#,
        city = BUSINESS.home_city,
        state = BUSINESS.state,
        phone_e164 = BUSINESS.phone_e164,
        phone = BUSINESS.phone,
        facebook = BUSINESS.facebook_url,
    )
}

fn breadcrumbs(items: &[(&'static str, String)]) -> String {
    if items.len() <= 1 {
        return String::new();
    }
    let list = items
        .iter()
        .map(|(name, path)| {
            format!(
                r#"<li><a href="{}">{}</a></li>"#,
                escape_attr(path),
                escape(name)
            )
        })
        .collect::<String>();
    format!(r#"<nav class="breadcrumbs" aria-label="Breadcrumb"><ol>{list}</ol></nav>"#)
}

fn contact_section(title: &str) -> String {
    format!(
        r#"<section class="section contact" aria-labelledby="contact-block-title"><div class="contact-copy"><p class="eyebrow">Contact</p><h2 id="contact-block-title">{title}</h2><a href="mailto:{email}">{email}</a><a href="tel:{phone_e164}">{phone}</a><a href="{facebook}" rel="noreferrer">Facebook</a></div>{form}</section>"#,
        title = escape(title),
        email = BUSINESS.email,
        phone_e164 = BUSINESS.phone_e164,
        phone = BUSINESS.phone,
        facebook = BUSINESS.facebook_url,
        form = contact_form(),
    )
}

fn contact_form() -> String {
    r#"<form aria-label="Pet service inquiry form" method="post" action="/api/inquiries">
<label for="name">Name<input id="name" name="name" autocomplete="name" required></label>
<label for="email">Email<input id="email" name="email" type="email" autocomplete="email" required></label>
<label for="phone">Phone<input id="phone" name="phone" type="tel" autocomplete="tel"></label>
<label for="preferred_contact_method">Preferred contact method<select id="preferred_contact_method" name="preferred_contact_method"><option>Email</option><option>Phone</option><option>Text</option></select></label>
<label for="city_or_zip">City or ZIP code<input id="city_or_zip" name="city_or_zip" autocomplete="postal-code" required></label>
<label for="pet_name">Pet name<input id="pet_name" name="pet_name"></label>
<label for="pet_age">Pet age<input id="pet_age" name="pet_age"></label>
<label for="service_of_interest">Service of interest<select id="service_of_interest" name="service_of_interest"><option>Dog training</option><option>Puppy training</option><option>Group dog classes</option><option>Not sure</option></select></label>
<label for="preferred_timeframe">Preferred timeframe<input id="preferred_timeframe" name="preferred_timeframe"></label>
<label class="wide" for="message">Goals or needs<textarea id="message" name="message" required></textarea></label>
<label class="wide consent" for="consent_acknowledged"><input id="consent_acknowledged" name="consent_acknowledged" type="checkbox" required> I consent to Cooper &amp; Co. using this information to respond to my inquiry.</label>
<label class="hp" for="website">Website<input id="website" name="website" tabindex="-1" autocomplete="off"></label>
<p class="privacy-note wide">Do not submit emergency, financial, or private medical information through this form.</p>
<button class="button primary" type="submit">Send inquiry</button>
<p class="form-status" role="status" aria-live="polite"></p>
</form>"#
        .to_owned()
}

fn service_card(service: &ServiceDefinition) -> String {
    format!(
        r#"<article class="card"><h3><a href="/services/{slug}">{name}</a></h3><p>{summary}</p><a href="/services/{slug}">Learn about {name}</a></article>"#,
        slug = service.slug,
        name = escape(service.name),
        summary = escape(service.summary)
    )
}

fn resource_card(article: &ResourceArticle) -> String {
    format!(
        r#"<article class="card"><span class="card-label">Resource</span><h3><a href="/resources/{slug}">{title}</a></h3><p>{description}</p><a href="/resources/{slug}">Read article</a></article>"#,
        slug = article.slug,
        title = escape(article.title),
        description = escape(article.description)
    )
}

fn list_markup(items: &[&str]) -> String {
    let items = items
        .iter()
        .map(|item| format!("<li>{}</li>", escape(item)))
        .collect::<String>();
    format!("<ul>{items}</ul>")
}

fn faq_markup(items: &[FaqItem]) -> String {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            format!(
                r#"<details {open}><summary>{question}</summary><p>{answer}</p></details>"#,
                open = if index == 0 { "open" } else { "" },
                question = escape(item.question),
                answer = escape(item.answer)
            )
        })
        .collect()
}

fn organization_schema() -> Value {
    let mut schema = json!({
        "@type": ["Organization", "LocalBusiness", "ProfessionalService"],
        "@id": format!("{}/#organization", canonical_origin()),
        "name": BUSINESS.name,
        "url": format!("{}/", canonical_origin()),
        "telephone": BUSINESS.phone_e164,
        "email": BUSINESS.email,
        "image": format!("{}{}", canonical_origin(), SOCIAL_IMAGE),
        "areaServed": [format!("{}, {}", BUSINESS.county, BUSINESS.state)],
        "address": {
            "@type": "PostalAddress",
            "addressLocality": BUSINESS.home_city,
            "addressRegion": "OH",
            "addressCountry": "US"
        },
        "sameAs": [BUSINESS.facebook_url, BUSINESS.yelp_url]
    });

    if let Some(public_address) = BUSINESS.public_address {
        schema["address"]["streetAddress"] = json!(public_address);
    }

    schema
}

fn local_business_schema() -> Value {
    organization_schema()
}

fn website_schema() -> Value {
    json!({
        "@type": "WebSite",
        "@id": format!("{}/#website", canonical_origin()),
        "url": format!("{}/", canonical_origin()),
        "name": BUSINESS.name,
        "publisher": {"@id": format!("{}/#organization", canonical_origin())}
    })
}

fn webpage_schema(path: &str, schema_type: &str) -> Value {
    json!({
        "@type": schema_type,
        "@id": format!("{}{}#webpage", canonical_origin(), path),
        "url": format!("{}{}", canonical_origin(), path),
        "isPartOf": {"@id": format!("{}/#website", canonical_origin())},
        "about": {"@id": format!("{}/#organization", canonical_origin())}
    })
}

fn service_schema(service: &ServiceDefinition) -> Value {
    json!({
        "@type": "Service",
        "@id": format!("{}/services/{}#service", canonical_origin(), service.slug),
        "name": service.name,
        "description": service.summary,
        "provider": {"@id": format!("{}/#organization", canonical_origin())},
        "areaServed": format!("{}, {}", BUSINESS.county, BUSINESS.state),
        "serviceType": service.name
    })
}

fn service_collection_schema() -> Value {
    json!({
        "@type": "ItemList",
        "@id": format!("{}/services#services", canonical_origin()),
        "itemListElement": SERVICES.iter().enumerate().map(|(index, service)| json!({
            "@type": "ListItem",
            "position": index + 1,
            "url": format!("{}/services/{}", canonical_origin(), service.slug),
            "name": service.name
        })).collect::<Vec<_>>()
    })
}

fn article_schema(article: &ResourceArticle) -> Value {
    json!({
        "@type": "Article",
        "@id": format!("{}/resources/{}#article", canonical_origin(), article.slug),
        "headline": article.title,
        "description": article.description,
        "author": {"@id": format!("{}/#organization", canonical_origin())},
        "publisher": {"@id": format!("{}/#organization", canonical_origin())},
        "datePublished": article.published,
        "dateModified": article.modified,
        "mainEntityOfPage": format!("{}/resources/{}", canonical_origin(), article.slug)
    })
}

fn confirmed_service_areas() -> Vec<&'static ServiceArea> {
    SERVICE_AREAS
        .iter()
        .filter(|area| area.status == LocationStatus::Confirmed)
        .collect()
}

fn canonical_origin() -> String {
    env::var("PRODUCTION_SITE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| PRODUCTION_ORIGIN.to_owned())
        .trim_end_matches('/')
        .to_owned()
}

fn staging_noindex_enabled() -> bool {
    env::var("COOPERCO_NOINDEX")
        .map(|value| truthy(&value))
        .unwrap_or(false)
        || ["PUBLIC_SITE_URL", "PUBLIC_APP_URL", "BACKEND_BASE_URL"]
            .iter()
            .filter_map(|name| env::var(name).ok())
            .any(|value| {
                let value = value.to_ascii_lowercase();
                value.contains("beta.") || value.contains("staging")
            })
}

fn verification_and_analytics_hooks() -> String {
    let mut hooks = String::new();
    if let Ok(value) = env::var("GOOGLE_SITE_VERIFICATION")
        .or_else(|_| env::var("COOPERCO_SEARCH_CONSOLE_VERIFICATION"))
    {
        if valid_token(&value) {
            hooks.push_str(&format!(
                r#"<meta name="google-site-verification" content="{}">"#,
                escape_attr(value.trim())
            ));
        }
    }
    if let Ok(value) = env::var("BING_SITE_VERIFICATION") {
        if valid_token(&value) {
            hooks.push_str(&format!(
                r#"<meta name="msvalidate.01" content="{}">"#,
                escape_attr(value.trim())
            ));
        }
    }
    if let Ok(value) = env::var("GA4_MEASUREMENT_ID").or_else(|_| env::var("GTM_CONTAINER_ID")) {
        let value = value.trim();
        if value.starts_with("G-") || value.starts_with("GTM-") {
            hooks.push_str(&format!(
                r#"<meta name="cooperco-analytics-id" content="{}">"#,
                escape_attr(value)
            ));
        }
    }
    if let Ok(value) = env::var("MICROSOFT_CLARITY_ID") {
        if valid_token(&value) {
            hooks.push_str(&format!(
                r#"<meta name="cooperco-clarity-id" content="{}">"#,
                escape_attr(value.trim())
            ));
        }
    }
    if let Ok(value) = env::var("META_PIXEL_ID") {
        if value.chars().all(|ch| ch.is_ascii_digit()) && !value.is_empty() {
            hooks.push_str(&format!(
                r#"<meta name="cooperco-meta-pixel-id" content="{}">"#,
                escape_attr(value.trim())
            ));
        }
    }
    hooks
}

fn valid_token(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}

fn obsolete_redirect(path: &str) -> Option<&'static str> {
    match path {
        "/service-area/lorain-oh" => Some("/service-areas/lorain-oh"),
        "/service-area" | "/service-area/" => Some("/service-areas"),
        "/service-area/mansfield-oh"
        | "/service-area/ontario-oh"
        | "/service-area/lexington-oh"
        | "/service-area/bellville-oh"
        | "/service-area/ashland-oh"
        | "/service-area/galion-oh"
        | "/service-areas/mansfield-oh"
        | "/service-areas/ontario-oh"
        | "/service-areas/lexington-oh"
        | "/service-areas/bellville-oh"
        | "/service-areas/ashland-oh"
        | "/service-areas/galion-oh" => Some("/service-areas"),
        _ => None,
    }
}

fn is_gone_path(path: &str) -> bool {
    matches!(
        path,
        "/services/dog-walking"
            | "/services/pet-sitting"
            | "/services/house-sitting"
            | "/services/puppy-care"
            | "/services/dog-adventures"
            | "/resources/local-dog-walking-checklist"
            | "/resources/puppy-care-first-week"
            | "/resources/dog-adventure-safety"
    )
}

fn normalize_path(path: &str) -> String {
    let without_query = path.split('?').next().unwrap_or(path);
    if without_query != "/" {
        without_query.trim_end_matches('/').to_owned()
    } else {
        "/".to_owned()
    }
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

fn truthy(value: &str) -> bool {
    matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "YES")
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(value: &str) -> String {
    escape(value).replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sitemap_uses_only_indexable_lorain_routes() {
        let sitemap = sitemap_body();
        assert!(sitemap.contains("https://cooper-and-co.com/service-areas/lorain-oh"));
        assert!(!sitemap.contains("beta.cooper-and-co.com"));
        assert!(!sitemap.contains("/admin"));
        assert!(!sitemap.contains("/api/"));
        assert!(!sitemap.contains("/auth/"));
        assert!(!sitemap.contains("mansfield"));
    }

    #[test]
    fn all_indexable_pages_have_unique_metadata_and_valid_json_ld() {
        let mut titles = std::collections::HashSet::new();
        let mut descriptions = std::collections::HashSet::new();
        for path in indexable_paths() {
            let page = page_for_path(&path).expect("route exists");
            assert!(titles.insert(page.title.clone()), "duplicate title {path}");
            assert!(
                descriptions.insert(page.description.clone()),
                "duplicate description {path}"
            );
            let html = render_page(&page);
            assert_eq!(html.matches("<h1").count(), 1, "one H1 on {path}");
            assert!(html.contains(r#"<link rel="canonical" href="https://cooper-and-co.com"#));
            for block in json_ld_blocks(&html) {
                serde_json::from_str::<Value>(&block).expect("valid json-ld");
            }
        }
    }

    #[test]
    fn registry_validation_passes() {
        assert_eq!(validation_errors(), Vec::<String>::new());
    }

    #[test]
    fn robots_changes_for_staging() {
        env::set_var("COOPERCO_NOINDEX", "true");
        assert_eq!(robots_body(), "User-agent: *\nDisallow: /\n");
        env::remove_var("COOPERCO_NOINDEX");
        assert!(robots_body().contains("Disallow: /admin"));
    }
}
