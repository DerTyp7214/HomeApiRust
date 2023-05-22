#![feature(decl_macro, proc_macro_hygiene)]

mod hue;
mod repsonses;
mod db {
    pub mod connection;
    pub mod huebridges;
    pub mod models;
    pub mod schema;
    pub mod users;
    pub mod usersettings;
    pub mod wleditems;
}

mod auth {
    pub mod auth;
    pub mod routes;
}

use std::path::Path;

use okapi::openapi3::OpenApi;
use rocket::{
    catch, catchers,
    fs::FileServer,
    get,
    response::Redirect,
    routes,
    serde::{self, json::Json},
    Build, Rocket,
};
use rocket_okapi::{
    mount_endpoints_and_merged_docs, openapi,
    settings::{OpenApiSettings, UrlObject},
    JsonSchema,
};
use rocket_okapi::{
    openapi_get_routes,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};
use schemars::gen::SchemaSettings;

use crate::db::connection;

#[derive(serde::Serialize, JsonSchema)]
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

#[catch(400)]
fn bad_request() -> Json<&'static str> {
    Json("Bad Request")
}

#[catch(401)]
fn unauthorized() -> Json<&'static str> {
    Json("Unauthorized")
}

#[catch(404)]
fn not_found() -> Json<&'static str> {
    Json("Not Found")
}

#[catch(409)]
fn conflict() -> Json<&'static str> {
    Json("Conflict")
}

#[catch(500)]
fn internal_error() -> Json<&'static str> {
    Json("Internal Server Error")
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

    let dist = Path::new("dist");
    if dist.exists() {
        api = api.mount("/static", FileServer::from(dist));
    }

    api = api
        .manage(connection::establish_connection())
        .mount("/", routes![redirect])
        .register(
            "/",
            catchers![
                bad_request,
                unauthorized,
                not_found,
                conflict,
                internal_error
            ],
        );

    let openapi_settings = &mut OpenApiSettings {
        schema_settings: SchemaSettings::openapi3(),
        ..Default::default()
    };
    let docs_route_spec = route_spec![make_swagger_ui(&SwaggerUIConfig {
        urls: vec![
            UrlObject {
                name: "API".to_string(),
                url: "/api/openapi.json".to_owned(),
            },
            UrlObject {
                name: "Auth".to_string(),
                url: "/api/auth/openapi.json".to_owned(),
            },
            UrlObject {
                name: "Hue".to_string(),
                url: "/api/hue/openapi.json".to_owned(),
            }
        ],
        deep_linking: true,
        ..Default::default()
    })];

    mount_endpoints_and_merged_docs! {
        api,
        "/".to_owned(),
        openapi_settings,
        "/api" => route_spec![openapi_get_routes![openapi_settings: status]],
        "/api/hue" => route_spec![hue::routes(openapi_settings)],
        "/api/auth" => route_spec![auth::routes::routes(openapi_settings)],
        "/docs" => docs_route_spec,
    };

    api
}

#[rocket::main]
async fn main() {
    dotenv::dotenv().ok();
    let launch_result = create_server().launch().await;

    match launch_result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
