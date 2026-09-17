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
/// Fallback change date for pages that carry no date of their own. Update it
/// when the marketing copy or layout of those pages changes.
const SITE_LASTMOD: &str = "2026-09-17";
const SOCIAL_IMAGE: &str = "/assets/cooperco-pet-services-hero.webp";
const SOCIAL_IMAGE_ALT: &str =
    "Black and tan dog on a leash in a park with dog-training cones in the background";

#[derive(Clone, Copy, Debug)]
pub struct BusinessProfile {
    pub name: &'static str,
    pub phone: &'static str,
    pub phone_e164: &'static str,
    pub email: &'static str,
    pub home_city: &'static str,
    pub state: &'static str,
    pub county: &'static str,
    pub facebook_url: &'static str,
    pub yelp_url: &'static str,
}

/// An image published on a page. `basename` names an asset that exists in both
/// AVIF and WebP form under `/assets`.
#[derive(Clone, Copy, Debug)]
pub struct PageImage {
    pub basename: &'static str,
    pub alt: &'static str,
    pub width: u32,
    pub height: u32,
}

impl PageImage {
    fn webp(&self) -> String {
        format!("/assets/{}.webp", self.basename)
    }

    fn avif(&self) -> String {
        format!("/assets/{}.avif", self.basename)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ServiceDefinition {
    pub slug: &'static str,
    pub name: &'static str,
    pub page_title: &'static str,
    pub description: &'static str,
    pub summary: &'static str,
    /// A direct, self-contained answer to the question this page exists for,
    /// placed first in the main content. Extraction into a featured snippet or
    /// an AI Overview works from the opening passage.
    pub answer: &'static str,
    pub audience: &'static str,
    pub image: PageImage,
    pub process: &'static [&'static str],
    pub prepare: &'static [&'static str],
    pub faq: &'static [FaqItem],
    pub related_resources: &'static [&'static str],
}

#[derive(Clone, Copy, Debug)]
pub struct ServiceArea {
    pub slug: &'static str,
    pub name: &'static str,
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
    /// See `ServiceDefinition::answer`.
    pub answer: &'static str,
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
    facebook_url: "https://www.facebook.com/CooperAndCoPet",
    yelp_url: "https://www.yelp.com/biz/cooper-and-company-elyria",
};

pub const SERVICE_AREAS: &[ServiceArea] = &[
    ServiceArea {
        slug: "elyria-oh",
        name: "Elyria, OH",
    },
    ServiceArea {
        slug: "lorain-oh",
        name: "Lorain, OH",
    },
    ServiceArea {
        slug: "amherst-oh",
        name: "Amherst, OH",
    },
    ServiceArea {
        slug: "avon-oh",
        name: "Avon, OH",
    },
    ServiceArea {
        slug: "north-ridgeville-oh",
        name: "North Ridgeville, OH",
    },
];

/// Each service answers its own questions. Three services previously shared one
/// FaqItem list, which published a byte-identical FAQPage entity on three URLs.
const DOG_TRAINING_FAQ: &[FaqItem] = &[
    FaqItem {
        question: "What should a dog training inquiry include?",
        answer: "Send your dog's age, breed or size, current training experience, and the goals you have in mind, such as leash skills, focus, or household manners.",
    },
    FaqItem {
        question: "Should I mention triggers or health limits?",
        answer: "Yes. Note any known triggers, safety concerns, health limits, and veterinarian guidance. Medical questions should go to a qualified veterinarian.",
    },
    FaqItem {
        question: "What happens after I send a dog training inquiry?",
        answer: "Cooper & Co. reviews fit, timing, and the next step, then responds with current availability and preparation details.",
    },
];

const PUPPY_TRAINING_FAQ: &[FaqItem] = &[
    FaqItem {
        question: "What should a puppy training inquiry include?",
        answer: "Share your puppy's age, schedule, comfort around people or dogs, and any handling notes that affect training.",
    },
    FaqItem {
        question: "Which puppy topics can I ask about?",
        answer: "Puppy inquiries commonly cover potty routines, crate practice, leash exposure, and building focus without overwhelming a young dog.",
    },
    FaqItem {
        question: "Who should I ask about vaccinations or medical concerns?",
        answer: "A qualified veterinarian. Include any veterinarian guidance that affects participation when you send an inquiry.",
    },
];

const GROUP_CLASS_FAQ: &[FaqItem] = &[
    FaqItem {
        question: "How do I know a group class is a fit for my dog?",
        answer: "Describe your dog's age, temperament, and goals. Cooper & Co. confirms whether the current group format is appropriate before class.",
    },
    FaqItem {
        question: "What should I mention about behavior around other dogs?",
        answer: "Note any known reactivity, fear, overexcitement, or safety concerns so class fit can be assessed before you attend.",
    },
    FaqItem {
        question: "Are class times and openings listed on the website?",
        answer: "The website does not publish a class schedule. Ask about current class format, capacity, and requirements through the contact options.",
    },
];

pub const SERVICES: &[ServiceDefinition] = &[
    ServiceDefinition {
        slug: "dog-training",
        name: "Dog training",
        page_title: "Dog Training in Lorain County, Ohio | Cooper & Co.",
        description: "Ask Cooper & Co. about dog training in Lorain County, Ohio, covering leash manners, focus, and everyday skills. Serving Elyria, Lorain, Amherst and Avon.",
        summary: "Dog training inquiries can cover leash manners, focus, everyday skills, and current training goals.",
        answer: "Cooper & Co. offers dog training in Lorain County, Ohio, serving Elyria, Lorain, Amherst, Avon, and North Ridgeville. Inquiries cover leash manners, focus, and everyday household skills. Send your dog's age, current training experience, and goals, and Cooper & Co. responds with fit, timing, and the next step.",
        audience: "Appropriate for dog owners who want clearer expectations, practical skills, and help choosing a class or training path.",
        image: PageImage {
            basename: "cooperco-pet-services-hero",
            alt: SOCIAL_IMAGE_ALT,
            width: 1600,
            height: 900,
        },
        process: &[
            "Send an inquiry with your city, dog details, and training goals.",
            "Cooper & Co. reviews fit, timing, and the next step.",
            "You receive current availability and preparation details directly from the business.",
        ],
        prepare: &[
            "Dog age, breed or size, and current training experience.",
            "Goals such as leash skills, focus, household manners, or class readiness.",
            "Known triggers, safety notes, health limits, and veterinarian guidance when relevant.",
        ],
        faq: DOG_TRAINING_FAQ,
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
        description: "Ask Cooper & Co. about puppy training in Lorain County, Ohio, covering early manners, routines, and class readiness. Serving Elyria, Lorain and Amherst.",
        summary: "Puppy training inquiries focus on early manners, confidence, routines, and class readiness.",
        answer: "Cooper & Co. offers puppy training in Lorain County, Ohio, serving Elyria, Lorain, Amherst, Avon, and North Ridgeville. Inquiries focus on early manners, confidence, daily routines, and readiness for a class. Share your puppy's age, schedule, and handling notes, and Cooper & Co. confirms whether the current format fits.",
        audience: "Appropriate for puppy owners who want early guidance without overwhelming a young dog.",
        image: PageImage {
            basename: "puppy-training-lorain-county",
            alt: "Golden puppy in a teal collar sitting on grass, watching a kneeling handler beside a treat pouch",
            width: 1200,
            height: 1200,
        },
        process: &[
            "Share puppy age, relevant health details, and goals.",
            "Cooper & Co. confirms whether the current format is a fit.",
            "You receive current preparation details before attending a class or session.",
        ],
        prepare: &[
            "Puppy age, schedule, comfort around people or dogs, and handling notes.",
            "Questions about potty routines, crate practice, leash exposure, or focus.",
            "Veterinarian guidance for medical or vaccination concerns.",
        ],
        faq: PUPPY_TRAINING_FAQ,
        related_resources: &[
            "preparing-your-puppy-for-its-first-training-class",
            "puppy-socialization-without-overwhelming-your-puppy",
            "what-to-bring-to-a-dog-training-class",
        ],
    },
    ServiceDefinition {
        slug: "group-dog-classes",
        name: "Group dog classes",
        page_title: "Group Dog Classes in Lorain County | Cooper & Co.",
        description: "Ask Cooper & Co. about group dog classes in Lorain County, Ohio, including class fit, what to prepare, and current availability. Send your dog's details.",
        summary: "Group dog class inquiries help determine class fit, readiness, goals, and current openings.",
        answer: "Cooper & Co. runs group dog classes in Lorain County, Ohio, serving Elyria, Lorain, Amherst, Avon, and North Ridgeville. Classes give dogs structured practice around other dogs and people. Describe your dog's age, temperament, and goals, and Cooper & Co. confirms whether the current group format is appropriate.",
        audience: "Appropriate for owners who want structured practice around other dogs and people when group settings are a fit.",
        image: PageImage {
            basename: "group-dog-classes-lorain-county",
            alt: "Five dogs sitting on leash beside their handlers on grass during an outdoor group class",
            width: 1200,
            height: 1200,
        },
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
        faq: GROUP_CLASS_FAQ,
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
        description: "A practical overview of how a group dog class runs, what to prepare beforehand, and the training expectations that are realistic for a first class.",
        answer: "A group dog training class usually opens with introductions, moves through short skill demonstrations, gives handlers practice time, and builds in breaks so dogs can reset. Expect steady progress over several sessions rather than a finished skill on the first day.",
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
        description: "Help your puppy arrive ready for a first training class, with simple planning, handling practice, and the health questions worth asking a veterinarian first.",
        answer: "Prepare a puppy for a first class by settling routines at home, practising gentle handling, and confirming health questions with a veterinarian first. Arrive with the gear the trainer asks for, and treat the first session as an introduction rather than a test.",
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
        description: "Simple leash-skill ideas dog owners in Lorain County can practice at home, in short sessions, before asking Cooper & Co. about training support.",
        answer: "Practise leash skills at home in short sessions, in a quiet room or yard, before adding distractions. Reward the dog for staying near you, keep the leash loose, and stop while the dog is still succeeding rather than pushing to the point of frustration.",
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
        description: "Questions Lorain County dog owners can ask when comparing a trainer, class, or training program, and the answers worth confirming before you commit.",
        answer: "Choose a dog trainer in Lorain County by asking how they handle your dog's specific goals, what a session looks like, how they respond when a dog struggles, and what they expect you to practise between sessions. Confirm the answers before you commit.",
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
        description: "Use these practical questions to decide whether a group dog class is a safe and useful fit for your dog, and what to confirm with the trainer beforehand.",
        answer: "Before joining a group dog class, ask what the class covers, how many dogs attend, what the space is like, what the trainer expects handlers to do, and how dogs that struggle are supported. The answers tell you whether the format suits your dog.",
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
        description: "A calm approach to puppy socialization that puts confidence, safety, and veterinarian guidance ahead of exposure for its own sake. Read it before class.",
        answer: "Socialise a puppy by keeping exposures short, calm, and optional, letting the puppy choose to approach rather than being carried into a crowd. Watch for signs of stress, end on a good note, and follow veterinarian guidance on where and when it is safe to go.",
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
        description: "Practical ways to build a dog's focus around everyday distractions, in short sessions and realistic settings, without expecting instant results.",
        answer: "Build focus around distractions by starting further away than you think you need, rewarding attention before the dog reacts, and shortening sessions as difficulty rises. Progress comes from many easy repetitions rather than a single hard one.",
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
        description: "A simple packing list for a dog training or puppy class, plus the handling notes and questions worth confirming with the trainer before you attend.",
        answer: "Bring a flat collar or harness, a standard leash, the rewards your dog actually works for, water, and any handling notes the trainer should know. Confirm the specific requirements with the trainer before the first session rather than guessing.",
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
        description: "Set practical dog training goals that account for how much you can practice, the environment, your dog's age, and the safety limits that apply.",
        answer: "Set dog training goals around how much you can practise, where the dog will need the skill, the dog's age, and any safety limits. A goal you can rehearse a few minutes a day beats an ambitious one that never gets practised.",
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
        description: "Low-pressure indoor enrichment ideas for cold Ohio winters, for the weeks when outdoor practice is limited and a dog still needs something to work on.",
        answer: "When Ohio winters limit outdoor practice, use indoor enrichment such as scent games, food puzzles, short skill sessions, and calm handling practice. A few focused minutes several times a day keeps a dog occupied without needing space or good weather.",
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
        fingerprinted: bool,
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
            MarketingResponse::Xml(body) => {
                let response = RawXml(body).respond_to(request)?;
                let mut builder = Response::build_from(response);
                builder.raw_header("Cache-Control", "public, max-age=3600");
                if staging_noindex_enabled() {
                    builder.header(Header::new("X-Robots-Tag", "noindex, nofollow"));
                }
                builder.ok()
            }
            MarketingResponse::Text(body) => {
                let mut response = Response::build();
                response.status(Status::Ok);
                response.header(ContentType::Plain);
                response.raw_header("Cache-Control", "public, max-age=3600");
                if staging_noindex_enabled() {
                    response.header(Header::new("X-Robots-Tag", "noindex, nofollow"));
                }
                response.sized_body(body.len(), std::io::Cursor::new(body));
                response.ok()
            }
            MarketingResponse::Redirect(redirect) => redirect.respond_to(request),
            MarketingResponse::File {
                file,
                x_robots,
                fingerprinted,
            } => {
                let response = file.respond_to(request)?;
                let mut builder = Response::build_from(response);
                // Only a URL that changes when its bytes change may be cached
                // immutably. Everything else must revalidate, or a deploy will
                // not reach browsers and edge caches that already hold it.
                builder.raw_header(
                    "Cache-Control",
                    if fingerprinted {
                        "public, max-age=31536000, immutable"
                    } else {
                        "public, max-age=3600, must-revalidate"
                    },
                );
                if x_robots {
                    builder.header(Header::new("X-Robots-Tag", "noindex, nofollow"));
                }
                builder.ok()
            }
        }
    }
}

/// The request path exactly as it arrived. Rocket's `PathBuf` segments drop a
/// trailing empty segment, so `/contact/` and `/contact` are indistinguishable
/// by the time a handler runs; redirecting duplicates needs the raw URI.
pub struct RawPath(String);

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for RawPath {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        rocket::request::Outcome::Success(RawPath(request.uri().path().as_str().to_owned()))
    }
}

#[get("/")]
pub async fn home_page() -> MarketingResponse {
    render_marketing_path("/").await
}

#[get("/<path..>", rank = 20)]
pub async fn marketing_page(path: PathBuf, raw: RawPath) -> MarketingResponse {
    if let Some(target) = duplicate_url_redirect(&raw.0) {
        return MarketingResponse::Redirect(Redirect::moved(target));
    }
    let path = format!("/{}", path.to_string_lossy().replace('\\', "/"));
    render_marketing_path(&path).await
}

/// Collapses URLs that serve identical content onto one address: a trailing
/// slash, and the build's `index.html`, which would otherwise answer with the
/// client-rendered shell as a thin duplicate of `/`.
fn duplicate_url_redirect(raw_path: &str) -> Option<String> {
    if raw_path == "/index.html" {
        return Some("/".to_owned());
    }
    if raw_path.len() > 1 && raw_path.ends_with('/') {
        let trimmed = raw_path.trim_end_matches('/');
        if trimmed.is_empty() {
            return Some("/".to_owned());
        }
        return Some(trimmed.to_owned());
    }
    None
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
        return MarketingResponse::Redirect(Redirect::moved(normalized));
    }

    if normalized == "/admin" || normalized.starts_with("/admin/") {
        return MarketingResponse::Html {
            status: Status::Ok,
            body: admin_shell_html(),
            x_robots: true,
        };
    }

    if let Some(target) = obsolete_redirect(&normalized) {
        return MarketingResponse::Redirect(Redirect::moved(target));
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
                    fingerprinted: false,
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
            fingerprinted: is_fingerprinted(relative),
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
    if staging_noindex_enabled() {
        return r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"></urlset>"#
            .to_owned();
    }

    let urls = indexable_paths()
        .iter()
        .map(|path| {
            let (changefreq, priority) = crawl_hints(path);
            format!(
                "<url><loc>{}{}</loc><lastmod>{}</lastmod><changefreq>{}</changefreq><priority>{}</priority></url>",
                canonical_origin(),
                path,
                lastmod_for(path),
                changefreq,
                priority
            )
        })
        .collect::<String>();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">{urls}</urlset>"#
    )
}

/// Resource pages carry their own modification date; everything else falls back
/// to the site-wide date. A single hard-coded date across every URL tells a
/// crawler nothing about which pages actually changed.
pub fn lastmod_for(path: &str) -> &'static str {
    path.strip_prefix("/resources/")
        .and_then(|slug| ARTICLES.iter().find(|article| article.slug == slug))
        .map(|article| article.modified)
        .unwrap_or(SITE_LASTMOD)
}

fn crawl_hints(path: &str) -> (&'static str, &'static str) {
    match path {
        "/" => ("monthly", "1.0"),
        "/privacy" | "/accessibility" => ("yearly", "0.5"),
        _ => ("monthly", "0.8"),
    }
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
                let _ = slug;
                return None;
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
        let rendered = render_page(&page);
        for blocked in [
            "TODO",
            "placeholder",
            "example.com",
            "owner-confirm",
            "owner approval",
            "owner-approved",
            "repository evidence",
            "candidate",
            "unconfirmed",
            "Mansfield",
            "Ontario",
            "Lexington",
            "Bellville",
            "Ashland",
            "Galion",
            "Pet Waste Removal",
            "Dog Walking",
            "Pet Sitting",
            "House Sitting",
            "Dog Adventures",
        ] {
            if rendered
                .to_ascii_lowercase()
                .contains(&blocked.to_ascii_lowercase())
            {
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
    <h1 id="home-title">Cooper &amp; Co. dog training and pet services in Lorain County</h1>
    <p>Cooper &amp; Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville. Ask about dog training, puppy training, and group dog classes.</p>
    <div class="hero-actions"><a class="button primary" href="/contact">Request information</a><a class="button secondary" href="tel:{phone_e164}">{phone}</a></div>
  </div>
</section>
<section class="section" aria-labelledby="home-services"><div class="section-heading"><p class="eyebrow">Services</p><h2 id="home-services">Dog training and class inquiries</h2><p>Use the current service pages to share dog details, training goals, location, and preferred timing.</p></div><div class="service-grid">{services}</div></section>
<section class="section split" aria-labelledby="classes-overview"><div><p class="eyebrow">Classes</p><h2 id="classes-overview">Group and puppy training inquiries</h2><p>Use the contact options on this site for current class details and availability.</p></div><article class="update"><span>Current next step</span><h3>Share your goals before booking</h3><p>Use the inquiry form to describe your dog, location, goals, and preferred timeframe.</p><a href="/contact">Contact Cooper &amp; Co.</a></article></section>
<section class="section" aria-labelledby="area-overview"><div class="section-heading"><p class="eyebrow">Service Area</p><h2 id="area-overview">Serving Lorain County</h2><p>Cooper &amp; Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville.</p></div><div class="service-grid"><article class="card"><h3><a href="/service-areas">Lorain County</a></h3><p>Review the current service-area page before sending an inquiry.</p></article><article class="card"><h3><a href="/contact">Ask about your location</a></h3><p>Include your city or ZIP code so Cooper &amp; Co. can respond with current fit.</p></article><article class="card"><h3><a href="tel:{phone_e164}">{phone}</a></h3><p>Call or text the listed business phone number for direct contact.</p></article></div></section>
<section class="section" aria-labelledby="process"><div class="section-heading"><p class="eyebrow">Process</p><h2 id="process">How inquiries work</h2></div><div class="service-grid"><article class="card"><h3>1. Send details</h3><p>Provide your contact details, city or ZIP code, pet age, service interest, and goals.</p></article><article class="card"><h3>2. Confirm fit</h3><p>Cooper &amp; Co. can confirm availability, class fit, and any requirements directly.</p></article><article class="card"><h3>3. Plan next steps</h3><p>You receive the appropriate scheduling or follow-up path from the business.</p></article></div></section>
<section class="section trust-section" aria-labelledby="contact-options"><div class="section-heading"><p class="eyebrow">Contact</p><h2 id="contact-options">Use the listed contact options</h2><p>The website publishes Cooper &amp; Co.'s business name, Lorain County service area, phone number, email, Facebook page, and Yelp listing.</p></div></section>
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
            FaqItem { question: "Where does Cooper & Co. serve?", answer: "Cooper & Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville." },
            FaqItem { question: "Which services are published on the website?", answer: "The published service pages are dog training, puppy training, and group dog classes." },
            FaqItem { question: "How do I ask about my location?", answer: "Include your city or ZIP code in the inquiry form so Cooper & Co. can respond with current fit." },
        ]),
        contact_section = contact_section("Ask about dog training or classes"),
    );

    Page {
        path: "/".to_owned(),
        title: "Cooper & Co. | Dog Training in Lorain County, Ohio".to_owned(),
        description: "Cooper & Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville. Ask about dog training and classes.".to_owned(),
        h1: "Cooper & Co. dog training and pet services in Lorain County".to_owned(),
        body,
        breadcrumbs: vec![("Home", "/".to_owned())],
        // The homepage summarises questions that /faq answers in full; only that
        // page carries the FAQPage entity, so the two do not compete.
        schema: vec![webpage_schema("/", "WebPage")],
        indexable: true,
    }
}

fn about() -> Page {
    basic_page(
        "/about",
        "About Cooper & Co. | Dog Training in Lorain County",
        "Learn how to reach Cooper & Co. about dog training, puppy training, and group dog classes in Lorain County, Ohio, and what to include in a first inquiry.",
        "About Cooper & Co.",
        "Cooper & Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville. Use the listed phone, email, Facebook, Yelp, or inquiry form to ask about dog training, puppy training, and group dog classes.",
        "AboutPage",
    )
}

fn services_index() -> Page {
    let cards = SERVICES.iter().map(service_card).collect::<String>();
    let body = format!(
        r#"<section class="section page-hero" aria-labelledby="services-title"><p class="eyebrow">Services</p><h1 id="services-title">Dog training services from Cooper &amp; Co.</h1><p>Use these pages to share dog details, training goals, Lorain County location, and preferred timing.</p></section><section class="section" aria-labelledby="services-published"><div class="section-heading"><h2 id="services-published">Published services</h2></div><div class="service-grid">{cards}</div></section>{contact}"#,
        contact = contact_section("Ask which training option fits your dog")
    );
    Page {
        path: "/services".to_owned(),
        title: "Dog Training Services in Lorain County | Cooper & Co.".to_owned(),
        description: "Explore Cooper & Co. dog training, puppy training, and group dog class pages for Lorain County pet owners, then send the details of your dog and goals.".to_owned(),
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
        r#"<section class="section page-hero" aria-labelledby="service-title"><p class="eyebrow">Service</p><h1 id="service-title">{h1}</h1><p class="answer">{answer}</p><p>{summary}</p><div class="hero-actions"><a class="button primary" href="/contact">Request information</a><a class="button secondary on-light" href="tel:{phone_e164}">{phone}</a></div>{figure}</section>
<section class="section" aria-labelledby="service-fit"><div class="section-heading"><p class="eyebrow">Fit</p><h2 id="service-fit">Who this may help</h2><p>{audience}</p></div></section>
<section class="section split" aria-labelledby="service-process"><div><p class="eyebrow">Process</p><h2 id="service-process">Expected inquiry process</h2>{process}</div><div><p class="eyebrow">Prepare</p><h2>What to share</h2>{prepare}</div></section>
<section class="section" aria-labelledby="availability"><div class="section-heading"><p class="eyebrow">Availability</p><h2 id="availability">Lorain County service area</h2><p>Cooper &amp; Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville. Include your city or ZIP code when you ask about fit.</p></div><a class="button secondary on-light" href="/service-areas">View service area</a></section>
<section class="section faq" aria-labelledby="service-faq"><div class="section-heading"><p class="eyebrow">FAQ</p><h2 id="service-faq">Service questions</h2></div>{faq}</section>
<section class="section" aria-labelledby="related"><div class="section-heading"><p class="eyebrow">Resources</p><h2 id="related">Related resources</h2></div><div class="service-grid">{related}</div></section>
{contact}"#,
        h1 = escape(&service_h1(service)),
        answer = escape(service.answer),
        summary = escape(service.summary),
        figure = figure_markup(&service.image, true),
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
        h1: service_h1(service),
        body,
        breadcrumbs: vec![
            ("Home", "/".to_owned()),
            ("Services", "/services".to_owned()),
            (service.name, path.clone()),
        ],
        schema: vec![
            webpage_schema(&path, "WebPage"),
            service_schema(service),
            page_image_schema(&service.image),
            faq_schema(&path, service.faq),
        ],
        indexable: true,
    }
}

/// "Dog training" alone says nothing about where. The title already carries the
/// county, so the visible heading should too.
fn service_h1(service: &ServiceDefinition) -> String {
    format!(
        "{} in {}, {}",
        service.name, BUSINESS.county, BUSINESS.state
    )
}

fn service_areas_index() -> Page {
    let areas = SERVICE_AREAS
        .iter()
        .map(|area| format!("<li>{}</li>", escape(area.name)))
        .collect::<String>();
    let body = format!(
        r#"<section class="section page-hero" aria-labelledby="areas-title"><p class="eyebrow">Service Areas</p><h1 id="areas-title">Cooper &amp; Co. service area</h1><p>Cooper &amp; Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville.</p></section><section class="section"><div class="section-heading"><h2>Lorain County communities listed on this site</h2><p>Include your city or ZIP code when sending an inquiry.</p></div><ul>{areas}</ul></section>{contact}"#,
        areas = areas,
        contact = contact_section("Confirm service availability in your city")
    );
    Page {
        path: "/service-areas".to_owned(),
        title: "Service Areas in Lorain County | Cooper & Co.".to_owned(),
        description: "Cooper & Co. serves Lorain County, Ohio, including Elyria, Lorain, Amherst, Avon, and North Ridgeville. Include your city or ZIP code in your inquiry.".to_owned(),
        h1: "Cooper & Co. service area".to_owned(),
        body,
        breadcrumbs: vec![
            ("Home", "/".to_owned()),
            ("Service Areas", "/service-areas".to_owned()),
        ],
        schema: vec![webpage_schema("/service-areas", "WebPage")],
        indexable: true,
    }
}

fn resources_index() -> Page {
    let cards = ARTICLES.iter().map(resource_card).collect::<String>();
    let body = format!(
        r#"<section class="section page-hero" aria-labelledby="resources-title"><p class="eyebrow">Resources</p><h1 id="resources-title">Dog training resources</h1><p>Educational articles help owners prepare thoughtful questions before contacting Cooper &amp; Co. Medical concerns should be directed to a qualified veterinarian.</p></section><section class="section" aria-labelledby="resources-published"><div class="section-heading"><h2 id="resources-published">All articles</h2></div><div class="service-grid">{cards}</div></section>{contact}"#,
        contact = contact_section("Ask a dog training question")
    );
    Page {
        path: "/resources".to_owned(),
        title: "Dog Training Resources | Cooper & Co.".to_owned(),
        description: "Read Cooper & Co. articles on group classes, puppy preparation, leash skills, and training expectations before you ask about dog training in Lorain County.".to_owned(),
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
        r#"<article class="section page-hero resource-article" aria-labelledby="article-title"><p class="eyebrow">Cooper &amp; Co. Resource</p><h1 id="article-title">{title}</h1><p class="answer">{answer}</p><p>{description}</p><p><strong>By Cooper &amp; Co.</strong> Published <time datetime="{published}">{published}</time>; updated <time datetime="{modified}">{modified}</time>.</p><div class="article-body">{sections}<h2>When to ask for help</h2><p>Contact Cooper &amp; Co. with your dog details, goals, and location. For medical concerns, consult a qualified veterinarian.</p></div><div class="hero-actions"><a class="button primary" href="/contact">Contact Cooper &amp; Co.</a><a class="button secondary on-light" href="/services/{service_slug}">{service_name}</a></div></article><section class="section" aria-labelledby="related-articles"><div class="section-heading"><p class="eyebrow">Related</p><h2 id="related-articles">Related articles</h2></div><div class="service-grid">{related}</div></section>"#,
        title = escape(article.title),
        answer = escape(article.answer),
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
        title: brand_title(article.title),
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
        title: "Contact Cooper & Co. in Lorain County".to_owned(),
        description: "Contact Cooper & Co. by phone, email, Facebook, Yelp, or the inquiry form about dog training, puppy training, and group dog classes in Lorain County, Ohio.".to_owned(),
        h1: "Contact Cooper & Co.".to_owned(),
        body,
        breadcrumbs: vec![("Home", "/".to_owned()), ("Contact", "/contact".to_owned())],
        schema: vec![webpage_schema("/contact", "ContactPage")],
        indexable: true,
    }
}

fn faq_page() -> Page {
    let body = format!(
        r#"<section class="section page-hero" aria-labelledby="faq-title"><p class="eyebrow">FAQ</p><h1 id="faq-title">Cooper &amp; Co. questions</h1><p>Use these answers to decide what to include when contacting Cooper &amp; Co.</p></section><section class="section faq">{faq}</section>{contact}"#,
        faq = faq_markup(&[
            FaqItem { question: "What services are listed?", answer: "The website lists dog training, puppy training, and group dog classes." },
            FaqItem { question: "Where does Cooper & Co. serve?", answer: "Cooper & Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville." },
            FaqItem { question: "Are hours or prices published?", answer: "The website does not publish fixed hours or prices. Use the contact form, phone, or email for current details." },
            FaqItem { question: "What should I send?", answer: "Send your contact details, city or ZIP code, pet name and age, service interest, goals, and preferred timeframe." },
        ]),
        contact = contact_section("Still have a question?"),
    );
    Page {
        path: "/faq".to_owned(),
        title: "FAQ | Cooper & Co. Dog Training in Lorain County".to_owned(),
        description: "Find answers about Cooper & Co. dog training inquiries, Lorain County service area, contact details, and what to include.".to_owned(),
        h1: "Cooper & Co. questions".to_owned(),
        body,
        breadcrumbs: vec![("Home", "/".to_owned()), ("FAQ", "/faq".to_owned())],
        schema: vec![
            webpage_schema("/faq", "WebPage"),
            faq_schema("/faq", &[
                FaqItem { question: "What services are listed?", answer: "The website lists dog training, puppy training, and group dog classes." },
                FaqItem { question: "Where does Cooper & Co. serve?", answer: "Cooper & Co. serves Lorain County, including Elyria, Lorain, Amherst, Avon, and North Ridgeville." },
                FaqItem { question: "Are hours or prices published?", answer: "The website does not publish fixed hours or prices. Use the contact form, phone, or email for current details." },
                FaqItem { question: "What should I send?", answer: "Send your contact details, city or ZIP code, pet name and age, service interest, goals, and preferred timeframe." },
            ]),
        ],
        indexable: true,
    }
}

fn privacy() -> Page {
    basic_page(
        "/privacy",
        "Privacy Policy | Cooper & Co.",
        "Read how Cooper & Co. handles website inquiries, which contact and pet-service details the form collects, and what should never be sent through this website.",
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

/// Appends the brand only while the result still fits the ~60 characters a
/// result page shows. A truncated title loses its tail, which matters more than
/// a brand that already appears in the URL and the breadcrumb.
fn brand_title(title: &str) -> String {
    let suffix = format!(" | {}", BUSINESS.name);
    if title.chars().count() + suffix.chars().count() <= 60 {
        format!("{title}{suffix}")
    } else {
        title.to_owned()
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
    // Every page's graph must be self-contained: `@id` references such as an
    // Article's author/publisher only resolve when the node they point at is
    // present in the same document.
    let mut graph = vec![
        local_business_schema(),
        website_schema(),
        image_object_schema(),
    ];
    graph.extend(page.schema.clone());
    if page.breadcrumbs.len() > 1 {
        graph.push(breadcrumb_schema(&page.breadcrumbs));
    }
    // A page may name the site-wide hero as its own image; keep one node per @id.
    let mut seen = std::collections::HashSet::new();
    graph.retain(|node| match node.get("@id").and_then(Value::as_str) {
        Some(id) => seen.insert(id.to_owned()),
        None => true,
    });
    let schema = json!({
        "@context": "https://schema.org",
        "@graph": graph
    });
    let canonical_link = if page.indexable {
        format!(
            r#"<link rel="canonical" href="{}">"#,
            escape_attr(&canonical)
        )
    } else {
        String::new()
    };
    let hooks = verification_and_analytics_hooks();
    let inquiry_script = inquiry_form_script();
    format!(
        r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="{robots}">
<title>{title}</title>
<meta name="description" content="{description}">
{canonical_link}
<link rel="icon" type="image/png" href="/assets/favicon.png">
<link rel="stylesheet" href="/styles.css">
<meta name="theme-color" content="#285c4d">
<meta name="application-name" content="{site_name}">
<meta name="apple-mobile-web-app-title" content="{site_name}">
<meta name="geo.region" content="US-OH">
<meta name="geo.placename" content="Lorain County, Ohio">
<meta property="og:type" content="website">
<meta property="og:locale" content="en_US">
<meta property="og:site_name" content="{site_name}">
<meta property="og:title" content="{title}">
<meta property="og:description" content="{description}">
<meta property="og:url" content="{canonical}">
<meta property="og:image" content="{origin}{social_image}">
<meta property="og:image:alt" content="{social_alt}">
<meta property="og:image:type" content="image/webp">
<meta property="og:image:width" content="1600">
<meta property="og:image:height" content="900">
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
{inquiry_script}
</body>
</html>"##,
        robots = robots,
        title = escape_attr(&page.title),
        description = escape_attr(&page.description),
        canonical_link = canonical_link,
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
        inquiry_script = inquiry_script,
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
    r#"<form aria-label="Pet service inquiry form" method="post" action="/api/inquiries" data-inquiry-form aria-describedby="form-status">
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
<label class="wide consent" for="consent_acknowledged"><input id="consent_acknowledged" name="consent_acknowledged" type="checkbox" value="true" required> I consent to Cooper &amp; Co. using this information to respond to my inquiry.</label>
<label class="hp" for="website">Website<input id="website" name="website" tabindex="-1" autocomplete="off"></label>
<p class="privacy-note wide">Do not submit emergency, financial, or private medical information through this form.</p>
<button class="button primary" type="submit" data-submit-label="Send inquiry">Send inquiry</button>
<p id="form-status" class="form-status" role="status" aria-live="polite"></p>
</form>"#
        .to_owned()
}

fn inquiry_form_script() -> &'static str {
    r#"<script>
document.querySelectorAll("[data-inquiry-form]").forEach((form) => {
  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    if (form.dataset.submitting === "true") {
      return;
    }
    const status = form.querySelector(".form-status");
    const button = form.querySelector("button[type='submit']");
    const formData = new FormData(form);
    const payload = Object.fromEntries(formData.entries());
    payload.consent_acknowledged = form.querySelector("[name='consent_acknowledged']")?.checked === true;
    payload.website = form.querySelector("[name='website']")?.value || "";
    form.dataset.submitting = "true";
    if (button) {
      button.disabled = true;
      button.setAttribute("aria-busy", "true");
      button.textContent = "Sending...";
    }
    if (status) {
      status.textContent = "Sending inquiry...";
    }
    try {
      const response = await fetch(form.action, {
        method: "POST",
        headers: {"Content-Type": "application/json"},
        body: JSON.stringify(payload)
      });
      if (!response.ok) {
        const message = await response.text();
        throw new Error(message || `Request failed with status ${response.status}`);
      }
      form.reset();
      if (status) {
        status.textContent = "Inquiry sent. Cooper & Co. can respond using the contact details provided.";
      }
    } catch (error) {
      if (status) {
        status.textContent = `Could not send inquiry. ${error.message}`;
      }
    } finally {
      form.dataset.submitting = "false";
      if (button) {
        button.disabled = false;
        button.removeAttribute("aria-busy");
        button.textContent = button.dataset.submitLabel || "Send inquiry";
      }
    }
  });
});
</script>"#
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

fn local_business_schema() -> Value {
    json!({
        "@type": ["LocalBusiness", "PetService"],
        "@id": format!("{}/#organization", canonical_origin()),
        "name": BUSINESS.name,
        "url": format!("{}/", canonical_origin()),
        "telephone": BUSINESS.phone_e164,
        "email": BUSINESS.email,
        "image": {"@id": format!("{}{}#image", canonical_origin(), SOCIAL_IMAGE)},
        "areaServed": service_area_schema(),
        "sameAs": [BUSINESS.facebook_url, BUSINESS.yelp_url]
    })
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
        "areaServed": service_area_schema(),
        "serviceType": service.name,
        "image": {"@id": format!("{}{}#image", canonical_origin(), service.image.webp())}
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

fn faq_schema(path: &str, items: &[FaqItem]) -> Value {
    json!({
        "@type": "FAQPage",
        "@id": format!("{}{}#faq", canonical_origin(), path),
        "url": format!("{}{}", canonical_origin(), path),
        "mainEntity": items.iter().map(|item| json!({
            "@type": "Question",
            "name": item.question,
            "acceptedAnswer": {
                "@type": "Answer",
                "text": item.answer
            }
        })).collect::<Vec<_>>()
    })
}

fn breadcrumb_schema(items: &[(&'static str, String)]) -> Value {
    json!({
        "@type": "BreadcrumbList",
        "itemListElement": items.iter().enumerate().map(|(index, (name, path))| json!({
            "@type": "ListItem",
            "position": index + 1,
            "name": name,
            "item": format!("{}{}", canonical_origin(), path)
        })).collect::<Vec<_>>()
    })
}

/// Renders a `<picture>` that prefers AVIF and falls back to WebP. Explicit
/// width/height keep the box reserved before the bytes land, so the image
/// cannot shift the layout.
fn figure_markup(image: &PageImage, priority: bool) -> String {
    let loading = if priority {
        r#"fetchpriority="high""#
    } else {
        r#"loading="lazy""#
    };
    format!(
        r#"<figure class="media-figure"><picture><source srcset="{avif}" type="image/avif"><img src="{webp}" alt="{alt}" width="{width}" height="{height}" {loading} decoding="async"></picture></figure>"#,
        avif = image.avif(),
        webp = image.webp(),
        alt = escape_attr(image.alt),
        width = image.width,
        height = image.height,
        loading = loading,
    )
}

fn page_image_schema(image: &PageImage) -> Value {
    json!({
        "@type": "ImageObject",
        "@id": format!("{}{}#image", canonical_origin(), image.webp()),
        "url": format!("{}{}", canonical_origin(), image.webp()),
        "contentUrl": format!("{}{}", canonical_origin(), image.webp()),
        "caption": image.alt,
        "width": image.width,
        "height": image.height,
        "encodingFormat": "image/webp"
    })
}

fn image_object_schema() -> Value {
    json!({
        "@type": "ImageObject",
        "@id": format!("{}{}#image", canonical_origin(), SOCIAL_IMAGE),
        "url": format!("{}{}", canonical_origin(), SOCIAL_IMAGE),
        "contentUrl": format!("{}{}", canonical_origin(), SOCIAL_IMAGE),
        "caption": SOCIAL_IMAGE_ALT,
        "width": 1600,
        "height": 900,
        "encodingFormat": "image/webp"
    })
}

fn service_area_schema() -> Vec<Value> {
    let mut areas = vec![json!({
        "@type": "AdministrativeArea",
        "name": format!("{}, {}", BUSINESS.county, BUSINESS.state)
    })];
    areas.extend(SERVICE_AREAS.iter().map(|area| {
        json!({
            "@type": "City",
            "name": area.name,
            "containedInPlace": format!("{}, {}", BUSINESS.county, BUSINESS.state)
        })
    }));
    areas
}

fn canonical_origin() -> String {
    env::var("PRODUCTION_SITE_URL")
        .ok()
        .filter(|value| {
            let value = value.trim().to_ascii_lowercase();
            !value.is_empty() && !value.contains("beta.") && !value.contains("staging")
        })
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
    if matches!(path, "/service-area" | "/service-area/") {
        return Some("/service-areas");
    }

    if let Some(slug) = path
        .strip_prefix("/service-area/")
        .or_else(|| path.strip_prefix("/service-areas/"))
    {
        if SERVICE_AREAS.iter().any(|area| area.slug == slug) {
            return Some("/service-areas");
        }
    }

    None
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
    ) || path
        .strip_prefix("/service-area/")
        .or_else(|| path.strip_prefix("/service-areas/"))
        .is_some_and(obsolete_non_lorain_slug)
}

fn obsolete_non_lorain_slug(slug: &str) -> bool {
    matches!(
        slug,
        "mansfield-oh"
            | "ontario-oh"
            | "lexington-oh"
            | "bellville-oh"
            | "ashland-oh"
            | "galion-oh"
    )
}

/// True when a filename embeds a content hash, as Trunk's build output does
/// (`index-1a2b3c4d5e6f7890.js`). Such a URL changes whenever its bytes do, so
/// it is safe to cache forever. `styles.css` and the files under `/assets` do
/// not, so they must not be.
fn is_fingerprinted(path: &str) -> bool {
    let Some(name) = path.rsplit('/').next() else {
        return false;
    };
    let Some((stem, _extension)) = name.rsplit_once('.') else {
        return false;
    };
    stem.rsplit_once('-')
        .is_some_and(|(_, hash)| hash.len() >= 8 && hash.chars().all(|ch| ch.is_ascii_hexdigit()))
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
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        env::remove_var("COOPERCO_NOINDEX");
        env::remove_var("PUBLIC_APP_URL");
        env::remove_var("PUBLIC_SITE_URL");
        env::remove_var("BACKEND_BASE_URL");
        env::remove_var("PRODUCTION_SITE_URL");
        let sitemap = sitemap_body();
        assert!(sitemap.contains("https://cooper-and-co.com/service-areas"));
        assert!(!sitemap.contains("https://cooper-and-co.com/service-areas/lorain-oh"));
        assert!(!sitemap.contains("beta.cooper-and-co.com"));
        assert!(!sitemap.contains("/admin"));
        assert!(!sitemap.contains("/api/"));
        assert!(!sitemap.contains("/auth/"));
        assert!(!sitemap.contains("mansfield"));
    }

    #[test]
    fn all_indexable_pages_have_unique_metadata_and_valid_json_ld() {
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        env::remove_var("COOPERCO_NOINDEX");
        env::remove_var("PUBLIC_APP_URL");
        env::remove_var("PUBLIC_SITE_URL");
        env::remove_var("BACKEND_BASE_URL");
        env::remove_var("PRODUCTION_SITE_URL");
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
            assert!(html.contains(r#"<meta name="geo.placename" content="Lorain County, Ohio">"#));
            for block in json_ld_blocks(&html) {
                serde_json::from_str::<Value>(&block).expect("valid json-ld");
            }
        }
    }

    #[test]
    fn json_ld_includes_required_schema_types_without_address_fields() {
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        env::remove_var("COOPERCO_NOINDEX");
        env::remove_var("PUBLIC_APP_URL");
        env::remove_var("PUBLIC_SITE_URL");
        env::remove_var("BACKEND_BASE_URL");
        env::remove_var("PRODUCTION_SITE_URL");
        let home = render_page(&home());
        let graph = json_ld_blocks(&home)
            .into_iter()
            .next()
            .and_then(|block| serde_json::from_str::<Value>(&block).ok())
            .and_then(|value| value.get("@graph").cloned())
            .and_then(|value| value.as_array().cloned())
            .expect("json graph");
        let graph_text = serde_json::to_string(&graph).unwrap();
        for schema_type in ["LocalBusiness", "PetService", "WebSite", "ImageObject"] {
            assert!(graph_text.contains(schema_type), "{schema_type}");
        }
        assert!(
            !graph_text.contains("FAQPage"),
            "the homepage must not compete with /faq for the same FAQ entity"
        );
        assert!(json_ld_blocks(&render_page(&faq_page()))
            .join("")
            .contains("FAQPage"));
        assert!(graph_text.contains("Elyria, OH"));
        assert!(graph_text.contains("North Ridgeville, OH"));
        assert!(!graph_text.contains("PostalAddress"));
        assert!(!graph_text.contains("streetAddress"));
        assert!(!graph_text.contains("latitude"));
        assert!(!graph_text.contains("longitude"));

        let service = render_page(&service_page(&SERVICES[0]));
        let service_graph = json_ld_blocks(&service).join("");
        assert!(service_graph.contains(r#""@type":"Service""#));
        assert!(service_graph.contains(r#""@type":"FAQPage""#));

        let services = render_page(&services_index());
        assert!(json_ld_blocks(&services)
            .join("")
            .contains("BreadcrumbList"));
    }

    #[test]
    fn beta_configuration_keeps_production_canonical_and_empty_sitemap() {
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        env::set_var("PUBLIC_APP_URL", "https://beta.cooper-and-co.com");
        env::set_var("PRODUCTION_SITE_URL", "https://beta.cooper-and-co.com");
        let html = render_page(&home());
        assert!(html.contains(r#"<meta name="robots" content="noindex, nofollow">"#));
        assert!(html.contains(r#"<link rel="canonical" href="https://cooper-and-co.com/">"#));
        assert!(!html.contains(r#"href="https://beta.cooper-and-co.com/"#));

        let sitemap = sitemap_body();
        assert!(!sitemap.contains("<loc>"));
        assert!(!sitemap.contains("beta.cooper-and-co.com"));

        env::remove_var("PUBLIC_APP_URL");
        env::remove_var("PRODUCTION_SITE_URL");
    }

    #[test]
    fn every_page_graph_resolves_its_own_id_references() {
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        env::remove_var("COOPERCO_NOINDEX");
        env::remove_var("PUBLIC_APP_URL");
        env::remove_var("PUBLIC_SITE_URL");
        env::remove_var("BACKEND_BASE_URL");
        env::remove_var("PRODUCTION_SITE_URL");

        for path in indexable_paths() {
            let page = page_for_path(&path).expect("route");
            let rendered = render_page(&page);
            let graph = json_ld_blocks(&rendered)
                .into_iter()
                .next()
                .and_then(|block| serde_json::from_str::<Value>(&block).ok())
                .and_then(|value| value.get("@graph").cloned())
                .and_then(|value| value.as_array().cloned())
                .expect("json graph");

            let defined = graph
                .iter()
                .filter_map(|node| node.get("@id").and_then(Value::as_str))
                .map(str::to_owned)
                .collect::<std::collections::HashSet<_>>();

            let mut referenced = Vec::new();
            collect_id_references(&Value::Array(graph.clone()), &mut referenced);
            for reference in referenced {
                assert!(
                    defined.contains(&reference),
                    "{path} references {reference} but never defines it"
                );
            }
        }
    }

    /// Collects `{"@id": "..."}` reference objects, i.e. nodes that point at an
    /// entity without describing one themselves.
    fn collect_id_references(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::Array(items) => items
                .iter()
                .for_each(|item| collect_id_references(item, out)),
            Value::Object(map) => {
                let is_reference = map.len() == 1 && map.contains_key("@id");
                if is_reference {
                    if let Some(id) = map.get("@id").and_then(Value::as_str) {
                        out.push(id.to_owned());
                    }
                }
                map.values()
                    .for_each(|item| collect_id_references(item, out));
            }
            _ => {}
        }
    }

    #[test]
    fn service_pages_publish_a_sized_image_backed_by_real_assets() {
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        for service in SERVICES {
            let rendered = render_page(&service_page(service));
            assert!(
                rendered.contains(&format!(r#"src="{}""#, service.image.webp())),
                "{} renders no image",
                service.slug
            );
            assert!(
                rendered.contains(&format!(r#"srcset="{}""#, service.image.avif())),
                "{} offers no avif source",
                service.slug
            );
            assert!(
                rendered.contains(&format!(r#"alt="{}""#, escape_attr(service.image.alt))),
                "{} image has no alt text",
                service.slug
            );
            assert!(
                rendered.contains(&format!(
                    r#"width="{}" height="{}""#,
                    service.image.width, service.image.height
                )),
                "{} image has no intrinsic dimensions",
                service.slug
            );

            for extension in ["webp", "avif"] {
                let asset = format!(
                    "frontend/public/assets/{}.{extension}",
                    service.image.basename
                );
                let workspace = std::path::Path::new(&asset);
                let from_backend = std::path::PathBuf::from("..").join(&asset);
                assert!(
                    workspace.is_file() || from_backend.is_file(),
                    "missing asset {asset}"
                );
            }
        }
    }

    #[test]
    fn only_content_hashed_filenames_are_cached_immutably() {
        for hashed in [
            "index-1a2b3c4d5e6f7890.js",
            "index-0123456789abcdef.css",
            "dist/frontend-deadbeefcafe1234_bg-0123456789abcdef.wasm",
        ] {
            assert!(is_fingerprinted(hashed), "{hashed}");
        }
        for plain in [
            "styles.css",
            "assets/cooperco-pet-services-hero.webp",
            "assets/facebook-cooperco-gallery-1.webp",
            "assets/favicon.png",
            "robots.txt",
            "noextension",
        ] {
            assert!(!is_fingerprinted(plain), "{plain}");
        }
    }

    #[test]
    fn duplicate_urls_collapse_onto_one_address() {
        assert_eq!(
            duplicate_url_redirect("/contact/"),
            Some("/contact".to_owned())
        );
        assert_eq!(
            duplicate_url_redirect("/services/dog-training/"),
            Some("/services/dog-training".to_owned())
        );
        assert_eq!(duplicate_url_redirect("/index.html"), Some("/".to_owned()));
        assert_eq!(duplicate_url_redirect("/contact"), None);
        assert_eq!(duplicate_url_redirect("/"), None);
    }

    /// `frontend/public/` holds copies of robots.txt and sitemap.xml for
    /// deployments that serve the built `dist` directly. They had already
    /// drifted from what Rocket generates, so pin them together.
    #[test]
    fn static_fallbacks_match_the_generated_output() {
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        env::remove_var("COOPERCO_NOINDEX");
        env::remove_var("PUBLIC_APP_URL");
        env::remove_var("PUBLIC_SITE_URL");
        env::remove_var("BACKEND_BASE_URL");
        env::remove_var("PRODUCTION_SITE_URL");

        for name in ["robots.txt", "robots"] {
            let fallback = read_public_file(name);
            assert_eq!(
                fallback.trim(),
                robots_body().trim(),
                "frontend/public/{name} has drifted from robots_body()"
            );
        }

        let generated = url_entries(&sitemap_body());
        let fallback = url_entries(&read_public_file("sitemap.xml"));
        assert_eq!(
            fallback, generated,
            "frontend/public/sitemap.xml has drifted from sitemap_body()"
        );
    }

    fn read_public_file(name: &str) -> String {
        let relative = format!("frontend/public/{name}");
        std::fs::read_to_string(&relative)
            .or_else(|_| std::fs::read_to_string(std::path::PathBuf::from("..").join(&relative)))
            .unwrap_or_else(|error| panic!("read {relative}: {error}"))
    }

    /// Compares `<url>` entries rather than raw bytes, so the checked-in copy
    /// may stay pretty-printed while Rocket answers on a single line.
    fn url_entries(xml: &str) -> Vec<String> {
        let mut entries = Vec::new();
        let mut rest = xml;
        while let Some(start) = rest.find("<url>") {
            let after = &rest[start..];
            let Some(end) = after.find("</url>") else {
                break;
            };
            entries.push(after[.."</url>".len() + end].trim().to_owned());
            rest = &after[end..];
        }
        entries
    }

    #[test]
    fn no_faq_question_is_published_on_more_than_one_url() {
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        let mut origin = std::collections::HashMap::new();
        for path in indexable_paths() {
            let page = page_for_path(&path).expect("route");
            for schema in &page.schema {
                if schema.get("@type").and_then(Value::as_str) != Some("FAQPage") {
                    continue;
                }
                let questions = schema
                    .get("mainEntity")
                    .and_then(Value::as_array)
                    .expect("mainEntity");
                for question in questions {
                    let name = question
                        .get("name")
                        .and_then(Value::as_str)
                        .expect("question name")
                        .to_owned();
                    if let Some(other) = origin.insert(name.clone(), path.clone()) {
                        panic!("{name:?} is marked up on both {other} and {path}");
                    }
                }
            }
        }
        assert!(!origin.is_empty(), "no FAQ markup found at all");
    }

    /// Google truncates around 60 characters of title and 155 of description.
    /// Overshooting loses the tail; undershooting wastes the slot.
    #[test]
    fn titles_and_descriptions_fit_the_serp() {
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        let mut too_long = Vec::new();
        let mut too_short = Vec::new();
        for path in indexable_paths() {
            let page = page_for_path(&path).expect("route");
            let title = page.title.chars().count();
            let description = page.description.chars().count();
            if title > 60 {
                too_long.push(format!("title {title} on {path}: {}", page.title));
            }
            if !(120..=158).contains(&description) {
                let bucket = if description < 120 {
                    &mut too_short
                } else {
                    &mut too_long
                };
                bucket.push(format!("description {description} on {path}"));
            }
        }
        assert!(
            too_long.is_empty() && too_short.is_empty(),
            "over: {too_long:#?}\nunder: {too_short:#?}"
        );
    }

    #[test]
    fn every_page_has_one_h1_and_no_skipped_heading_levels() {
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        for path in indexable_paths() {
            let page = page_for_path(&path).expect("route");
            let levels = heading_levels(&render_page(&page));
            assert_eq!(
                levels.iter().filter(|level| **level == 1).count(),
                1,
                "{path} does not have exactly one h1"
            );
            for pair in levels.windows(2) {
                assert!(
                    pair[1] <= pair[0] + 1,
                    "{path} jumps from h{} to h{}",
                    pair[0],
                    pair[1]
                );
            }
        }
    }

    fn heading_levels(html: &str) -> Vec<u32> {
        let mut levels = Vec::new();
        let mut rest = html;
        while let Some(start) = rest.find("<h") {
            let after = &rest[start + 2..];
            let mut chars = after.chars();
            if let (Some(digit), Some(next)) = (chars.next(), chars.next()) {
                if let Some(level) = digit.to_digit(10) {
                    if (1..=6).contains(&level) && (next == '>' || next == ' ') {
                        levels.push(level);
                    }
                }
            }
            rest = after;
        }
        levels
    }

    /// The opening passage is what a featured snippet or an AI Overview lifts,
    /// so it has to answer the page's question on its own, at a length that
    /// survives extraction.
    #[test]
    fn service_and_article_pages_open_with_a_direct_answer() {
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        let answers = SERVICES
            .iter()
            .map(|service| (service.slug, service.answer))
            .chain(
                ARTICLES
                    .iter()
                    .map(|article| (article.slug, article.answer)),
            );

        for (slug, answer) in answers {
            let words = answer.split_whitespace().count();
            assert!(
                (35..=70).contains(&words),
                "{slug} answer is {words} words, outside 35-70"
            );
            assert!(
                answer.ends_with('.'),
                "{slug} answer is not a complete passage"
            );
        }

        for service in SERVICES {
            let rendered = render_page(&service_page(service));
            // Compare positions within the body: the summary also appears in the
            // head, as the Service node's description.
            let body = &rendered[rendered.find("<main").expect("main")..];
            let answer_at = body.find(&escape(service.answer)).expect("answer");
            let summary_at = body.find(&escape(service.summary)).expect("summary");
            assert!(
                answer_at < summary_at,
                "{} buries its answer below the summary",
                service.slug
            );
        }

        for article in ARTICLES {
            let rendered = render_page(&article_page(article));
            assert!(
                rendered.contains(&escape(article.answer)),
                "{} renders no answer",
                article.slug
            );
        }
    }

    #[test]
    fn registry_validation_passes() {
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        env::remove_var("COOPERCO_NOINDEX");
        env::remove_var("PUBLIC_APP_URL");
        env::remove_var("PUBLIC_SITE_URL");
        env::remove_var("BACKEND_BASE_URL");
        env::remove_var("PRODUCTION_SITE_URL");
        assert_eq!(validation_errors(), Vec::<String>::new());
    }

    #[test]
    fn robots_changes_for_staging() {
        let _guard = crate::ENV_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        env::remove_var("PUBLIC_APP_URL");
        env::remove_var("PUBLIC_SITE_URL");
        env::remove_var("BACKEND_BASE_URL");
        env::remove_var("PRODUCTION_SITE_URL");
        env::set_var("COOPERCO_NOINDEX", "true");
        assert_eq!(robots_body(), "User-agent: *\nDisallow: /\n");
        assert!(!sitemap_body().contains("<loc>"));
        env::remove_var("COOPERCO_NOINDEX");
        assert!(robots_body().contains("Disallow: /admin"));
    }
}
