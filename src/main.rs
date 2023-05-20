#![feature(decl_macro, proc_macro_hygiene)]

use std::path::Path;

use okapi::openapi3::OpenApi;
use rocket::{
    fs::FileServer,
    get,
    response::Redirect,
    routes,
    serde::{self, json::Json},
    Build, Rocket,
};
use rocket_db_pools::{sqlx, Database};
use rocket_okapi::{
    mount_endpoints_and_merged_docs, openapi,
    settings::{OpenApiSettings, UrlObject},
    JsonSchema,
};
use rocket_okapi::{
    openapi_get_routes,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};

#[derive(Database)]
#[database("main")]
struct DbConn(sqlx::SqlitePool);

#[derive(serde::Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
struct Status {
    status: String,
    version: String,
}

#[get("/")]
fn redirect() -> Redirect {
    rocket::response::Redirect::to("/static")
}

#[openapi]
#[get("/status")]
fn status() -> Json<Status> {
    let status = Status {
        status: "OK".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Json(status)
}

fn custom_openapi_spec() -> OpenApi {
    use rocket_okapi::okapi::openapi3::Info;

    OpenApi {
        openapi: OpenApi::default_version(),
        info: Info {
            title: "Rust API".to_string(),
            description: Some("Rust API".to_string()),
            version: env!("CARGO_PKG_VERSION").to_string(),
            ..Default::default()
        },
        ..Default::default()
    }
}

macro_rules! route_spec {
    ($routes:expr) => {
        ($routes, custom_openapi_spec())
    };
    ($routes:expr, $openapi_spec:ident) => {
        ($routes, $openapi_spec)
    };
}

fn create_server() -> Rocket<Build> {
    let mut api = rocket::build();
    api = api.attach(DbConn::init());

    let dist = Path::new("dist");
    if dist.exists() {
        api = api.mount("/static", FileServer::from(dist));
    }

    api = api.mount("/", routes![redirect]);

    let openapi_settings = OpenApiSettings::default();
    let api_route_spec = route_spec![openapi_get_routes![status]];
    let docs_route_spec = route_spec![make_swagger_ui(&SwaggerUIConfig {
        urls: vec![UrlObject {
            name: "API".to_string(),
            url: "/api/openapi.json".to_owned(),
        }],
        deep_linking: true,
        ..Default::default()
    })];

    mount_endpoints_and_merged_docs! {
        api,
        "/".to_owned(),
        openapi_settings,
        "/api" => api_route_spec,
        "/docs" => docs_route_spec,
    };

    api
}

#[rocket::main]
async fn main() {
    let launch_result = create_server().launch().await;

    match launch_result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
