use rocket::{get, http::Status, serde::json::Json, State};
use rocket_okapi::openapi;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    auth::auth::JWTToken,
    db::{
        connection::{self, SqlitePool, SqlitePooledConnection},
        models::User,
    },
    repsonses::CustomResponse,
};

use super::hue;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct NormalizedColor(pub u8, pub u8, pub u8);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct NormalizedLight {
    pub id: String,
    pub name: String,
    pub on: bool,
    pub brightness: f32,
    pub color: Vec<NormalizedColor>,
    pub reachable: bool,
    #[serde(rename = "type")]
    pub type_: String,
    pub model: String,
    pub manufacturer: String,
    pub uniqueid: String,
    pub swversion: String,
    pub productid: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct NormalizedPlug {
    pub id: String,
    pub name: String,
    pub on: bool,
    pub reachable: bool,
    #[serde(rename = "type")]
    pub type_: String,
    pub model: String,
    pub manufacturer: String,
    pub uniqueid: String,
    pub swversion: String,
    pub productid: Option<String>,
}

async fn get_lights(
    connection: &mut SqlitePooledConnection,
    user: User,
) -> Result<Vec<NormalizedLight>, CustomResponse> {
    let bridges = user.get_huebridges(connection);

    if bridges.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Bridges not found".to_string(),
        });
    }

    let bridges = bridges.unwrap();

    let mut lights = Vec::new();

    for bridge in bridges {
        let bridge_lights = hue::get_lights(&bridge).await;
        lights.extend(bridge_lights);
    }

    Ok(lights)
}

async fn get_plugs(
    connection: &mut SqlitePooledConnection,
    user: User,
) -> Result<Vec<NormalizedPlug>, CustomResponse> {
    let bridges = user.get_huebridges(connection);

    if bridges.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Bridges not found".to_string(),
        });
    }

    let bridges = bridges.unwrap();

    let mut plugs = Vec::new();

    for bridge in bridges {
        let bridge_plugs = hue::get_plugs(&bridge).await;
        plugs.extend(bridge_plugs);
    }

    Ok(plugs)
}

#[openapi(tag = "Main")]
#[get("/lights")]
pub async fn lights(
    jwt: JWTToken,
    pool: &State<SqlitePool>,
) -> Result<Json<Vec<NormalizedLight>>, CustomResponse> {
    let connection = &mut connection::get_connection(pool).unwrap();

    let user = User::get_user(connection, jwt.user_id);

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Unauthorized".to_string(),
        });
    }

    let user = user.unwrap();

    let response = get_lights(connection, user).await;

    if response.is_err() {
        return Err(response.err().unwrap());
    }

    Ok(Json(response.unwrap()))
}

#[openapi(tag = "Main")]
#[get("/plugs")]
pub async fn plugs(
    jwt: JWTToken,
    pool: &State<SqlitePool>,
) -> Result<Json<Vec<NormalizedPlug>>, CustomResponse> {
    let connection = &mut connection::get_connection(pool).unwrap();

    let user = User::get_user(connection, jwt.user_id);

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Unauthorized".to_string(),
        });
    }

    let user = user.unwrap();

    let response = get_plugs(connection, user).await;

    if response.is_err() {
        return Err(response.err().unwrap());
    }

    Ok(Json(response.unwrap()))
}