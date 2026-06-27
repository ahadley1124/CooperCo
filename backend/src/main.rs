use std::{env, path::PathBuf, sync::Arc};

use anyhow::Context;
use rocket::{
    fs::{FileServer, NamedFile},
    get,
    http::Status,
    post,
    response::status,
    routes,
    serde::json::Json,
    Config, State,
};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};
use tokio::sync::RwLock;
use uuid::Uuid;

type Db = Surreal<Client>;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Inquiry {
    id: Uuid,
    name: String,
    email: String,
    phone: String,
    pet_name: String,
    message: String,
}

#[derive(Debug, Clone, Deserialize)]
struct NewInquiry {
    name: String,
    email: String,
    phone: String,
    pet_name: String,
    message: String,
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

#[get("/<_..>", rank = 20)]
async fn spa_fallback() -> Option<NamedFile> {
    NamedFile::open(static_dir().join("index.html")).await.ok()
}

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    let store = connect_store().await;

    rocket::build()
        .configure(Config {
            port: 8080,
            ..Config::debug_default()
        })
        .manage(store)
        .mount(
            "/",
            routes![health, site_content, create_inquiry, spa_fallback],
        )
        .mount("/", FileServer::from(static_dir()).rank(10))
        .launch()
        .await
        .context("rocket failed")?;

    Ok(())
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
                if let Err(error) = db
                    .signin(Root {
                        username: &username,
                        password: &password,
                    })
                    .await
                {
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

fn server_error(error: surrealdb::Error) -> status::Custom<String> {
    status::Custom(Status::InternalServerError, error.to_string())
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
            intro: "Pet support for families in Lorain County, with class updates and booking handled directly by Cooper & Co.".to_owned(),
            hero_image: "https://scontent-ord5-3.xx.fbcdn.net/v/t39.30808-6/642365926_122126318187116749_1263209954982424911_n.jpg?stp=dst-jpg_tt6&cstp=mx850x315&ctp=s850x315&_nc_cat=109&ccb=1-7&_nc_sid=cc71e4&_nc_ohc=YgBZS2Y2cIIQ7kNvwE6wWc1&_nc_oc=Adp6qZnwOPKmsH-P7WLn-Qi3NAm4iRj8LUXeorKSU7TZGtxYSmmUzQHHVLhjI4uO67U&_nc_zt=23&_nc_ht=scontent-ord5-3.xx&_nc_gid=mvj5NsrB-bhghI4HuF5-1A&_nc_ss=7b289&oh=00_Af8uQAkKPQNHbSMJT_M1dcsc77LWXluS0oRmbUhibqkllg&oe=6A452862".to_owned(),
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
                title: "Group classes".to_owned(),
                summary: "Seasonal class announcements and availability are highlighted from the public Facebook page.".to_owned(),
            },
            Service {
                title: "Pet service inquiries".to_owned(),
                summary: "A short contact flow captures pet details, owner contact information, and the help requested.".to_owned(),
            },
            Service {
                title: "Local support".to_owned(),
                summary: "Focused on pet families across Lorain County with direct phone and email contact.".to_owned(),
            },
        ],
        updates: vec![Update {
            title: "Summer group classes".to_owned(),
            summary: "The latest visible Facebook update promotes summer group classes. Contact Cooper & Co. for current times and openings.".to_owned(),
            source_label: "Facebook post, May 10".to_owned(),
        }],
        gallery: vec![
            GalleryImage {
                src: "https://scontent-ord5-1.xx.fbcdn.net/v/t39.30808-6/696722380_122136771933116749_5005000033412355237_n.jpg?stp=dst-jpg_tt6&cstp=mx1206x1206&ctp=s160x160&_nc_cat=111&ccb=1-7&_nc_sid=09d16d&_nc_ohc=JsAsHJCGwcMQ7kNvwFaTowy&_nc_oc=AdqsEPdRsmqSKvTm6-WMNV45aHS3X-kEpSG0w-NkvnKyZ_saGStYvRNGbor_GIIxm7Q&_nc_zt=23&_nc_ht=scontent-ord5-1.xx&_nc_gid=mvj5NsrB-bhghI4HuF5-1A&_nc_ss=7b289&oh=00_Af_8VRu4XN_x4tgpVo5giq-UBcRRY4QzMlZQ_26zTpCgSg&oe=6A450FB6".to_owned(),
                alt: "Cooper & Co. Facebook gallery item".to_owned(),
            },
            GalleryImage {
                src: "https://scontent-ord5-1.xx.fbcdn.net/v/t39.30808-6/697792713_122136770193116749_6164835244999125676_n.jpg?stp=c0.107.1206.1206a_cp6_dst-jpg_tt6&cstp=mx1206x1206&ctp=s160x160&_nc_cat=108&ccb=1-7&_nc_sid=8a6525&_nc_ohc=kJRpguVDutYQ7kNvwG80Nde&_nc_oc=Adqu1GzkMCpJRyZmMBeR5DNqKEOvPlbeOAvMCG6_cPPbFAe_9vwH7jpg-HbmtXW9PMQ&_nc_zt=23&_nc_ht=scontent-ord5-1.xx&_nc_gid=mvj5NsrB-bhghI4HuF5-1A&_nc_ss=7b289&oh=00_Af9X1NSjEEd5y2aPAu-gKNBVSxNt713qsUSJ-72h_53BXw&oe=6A452A12".to_owned(),
                alt: "Cooper & Co. Facebook gallery item".to_owned(),
            },
            GalleryImage {
                src: "https://scontent-ord5-2.xx.fbcdn.net/v/t39.30808-6/691710795_122136487347116749_7323625458125376568_n.jpg?stp=c0.119.1086.1086a_dst-jpg_tt6&cstp=mx1086x1086&ctp=s160x160&_nc_cat=104&ccb=1-7&_nc_sid=8a6525&_nc_ohc=7KGiODw34dgQ7kNvwEN1_rU&_nc_oc=AdrmEAKck0at6TOOBjGVeikPuV80YT9G8-__xhdoPIeLL1bztNkEJBgtJhxQ7J7xlZk&_nc_zt=23&_nc_ht=scontent-ord5-2.xx&_nc_gid=mvj5NsrB-bhghI4HuF5-1A&_nc_ss=7b289&oh=00_Af87OvLYcS9_HvmU3uE2dlz_-GZZe-Q_l_5g3rrq87QYlA&oe=6A453359".to_owned(),
                alt: "Cooper & Co. Facebook gallery item".to_owned(),
            },
        ],
    }
}
