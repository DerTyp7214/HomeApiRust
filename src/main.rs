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

use auth::auth::JWTToken;

use plugins::main::{NormalizedLight, NormalizedPlug};
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::error::RecvError;
use rocket::tokio::sync::broadcast::{channel, Sender};
use rocket::{
    catch, catchers, figment::Figment, fs::relative, fs::FileServer, get, response::Redirect,
    routes, serde::json::Json, Build, Rocket,
};
use rocket::{FromForm, Shutdown, State};
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_okapi::{
    mount_endpoints_and_merged_docs,
    settings::{OpenApiSettings, UrlObject},
};
use schemars::_serde_json;
use schemars::gen::SchemaSettings;
use serde::{Deserialize, Serialize};

use crate::db::connection;

#[get("/")]
fn redirect() -> Redirect {
    rocket::response::Redirect::to("/static")
}

#[get("/sse")]
pub async fn events(
    jwt: JWTToken,
    queue: &State<Sender<InternalMessage>>,
    mut end: Shutdown,
) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            if msg.token.user_id != jwt.user_id {
                yield Event::json(&Message {
                    _type: msg._type,
                    data: "".to_owned(),
                });
            } else {
                yield Event::json(&msg.to_message());
            }
        }
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

fn configure_rocket() -> Figment {
    rocket::Config::figment().merge((
        "secret_key",
        "ENDM3ymkXOrZHhK1Z2q8bLOaxZr3LTGm540bd6oXGheaX8KmltC+cSwnJ0b9zK7PFiaXPp+zFBF+BPnZF/htXw==",
    ))
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    _type: String,
    data: String,
}

#[derive(Debug, Clone)]
pub struct InternalMessage {
    _type: String,
    data: String,
    token: JWTToken,
}

impl InternalMessage {
    pub fn light_update(light: NormalizedLight, token: JWTToken) -> InternalMessage {
        InternalMessage {
            _type: "light_update".to_owned(),
            data: _serde_json::to_string(&light).unwrap(),
            token,
        }
    }
    pub fn plug_update(plug: NormalizedPlug, token: JWTToken) -> InternalMessage {
        InternalMessage {
            _type: "plug_update".to_owned(),
            data: _serde_json::to_string(&plug).unwrap(),
            token,
        }
    }

    pub fn to_message(&self) -> Message {
        Message {
            _type: self._type.clone(),
            data: self.data.clone(),
        }
    }
}

fn create_server() -> Rocket<Build> {
    let mut api = rocket::custom(configure_rocket());

    let dist = Path::new(relative!("dist"));
    if dist.exists() {
        api = api.mount("/static", FileServer::from(dist));
    }

    api = api
        .attach(cors::CORS)
        .manage(connection::establish_connection())
        .manage(channel::<InternalMessage>(1024).0)
        .mount("/", routes![redirect, events, cors::all_options])
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

    if std::env::var("MIGRATE").is_ok() {
        connection::run_migrations();
        return;
    }

    let launch_result = create_server().launch().await;

    match launch_result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
