#![feature(decl_macro, proc_macro_hygiene)]

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

mod plugins {
    pub mod assets;
    pub mod hue;
    pub mod main;
    pub mod user;
}

mod utils {
    pub mod color;
    pub mod extensions;
}

mod cors;

use std::path::Path;

use rocket::{
    catch, catchers, figment::Figment, fs::FileServer, get, response::Redirect, routes,
    serde::json::Json, Build, Rocket,
};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::{
    mount_endpoints_and_merged_docs,
    settings::{OpenApiSettings, UrlObject},
};
use schemars::gen::SchemaSettings;

use crate::db::connection;

#[get("/")]
fn redirect() -> Redirect {
    rocket::response::Redirect::to("/static")
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

fn configure_rocket() -> Figment {
    rocket::Config::figment().merge((
        "secret_key",
        "ENDM3ymkXOrZHhK1Z2q8bLOaxZr3LTGm540bd6oXGheaX8KmltC+cSwnJ0b9zK7PFiaXPp+zFBF+BPnZF/htXw==",
    ))
}

fn create_server() -> Rocket<Build> {
    let mut api = rocket::custom(configure_rocket());

    let dist = Path::new("dist");
    if dist.exists() {
        api = api.mount("/static", FileServer::from(dist));
    }

    api = api
        .attach(cors::CORS)
        .manage(connection::establish_connection())
        .mount("/", routes![redirect, cors::all_options])
        .mount(
            "/docs",
            make_swagger_ui(&SwaggerUIConfig {
                urls: vec![UrlObject {
                    name: "API".to_owned(),
                    url: "../openapi.json".to_owned(),
                }],
                deep_linking: true,
                ..Default::default()
            }),
        )
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

    let openapi_settings = OpenApiSettings {
        schema_settings: SchemaSettings::openapi3(),
        ..Default::default()
    };

    mount_endpoints_and_merged_docs! {
        api,
        "/".to_owned(),
        openapi_settings,
        "/api" => plugins::main::routes(&openapi_settings),
        "/api/user" => plugins::user::routes(&openapi_settings),
        "/api/hue" => plugins::hue::routes(&openapi_settings),
        "/api/auth" => auth::routes::routes(&openapi_settings),
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
