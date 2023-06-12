use okapi::openapi3::OpenApi;
use rocket::{get, http::Status, put, serde::json::Json, tokio::sync::broadcast::Sender, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use schemars::{
    JsonSchema,
    _serde_json::{json, Value},
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::auth::JWTToken,
    db::{
        connection::{self, SqlitePool, SqlitePooledConnection},
        models::User,
    },
    repsonses::CustomResponse,
    InternalMessage,
};

use super::hue;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct StatusResponse {
    status: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LightState {
    pub on: Option<bool>,
    pub brigthness: Option<u8>,
    pub color: Option<Vec<NormalizedColor>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PlugState {
    pub on: Option<bool>,
}

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

async fn get_light(
    connection: &mut SqlitePooledConnection,
    user: &User,
    light_id: &String,
) -> Result<NormalizedLight, CustomResponse> {
    let light_id = light_id.to_owned();
    let provider = light_id.split('-').nth(0).unwrap();

    if provider == "hue" {
        let bridge_id = light_id.split('-').nth(1).unwrap();
        let light_id = light_id.split('-').nth(2).unwrap();

        let bridge = user.get_huebridge(connection, bridge_id);

        if bridge.is_err() {
            return Err(CustomResponse {
                status: Status::NotFound,
                message: "Bridge not found".to_string(),
            });
        }

        let bridge = bridge.unwrap();

        let light = hue::get_light(&bridge, &light_id.to_owned()).await;

        if light.is_err() {
            return Err(CustomResponse {
                status: Status::NotFound,
                message: "Light not found".to_string(),
            });
        }

        let light = light.unwrap();

        return Ok(light);
    }

    Err(CustomResponse {
        status: Status::NotFound,
        message: "Unknown provider".to_string(),
    })
}

async fn set_light_state(
    connection: &mut SqlitePooledConnection,
    user: &User,
    light_id: &String,
    state: LightState,
) -> Result<(), CustomResponse> {
    let light_id = light_id.to_owned();
    let provider = light_id.split('-').nth(0).unwrap();

    if provider == "hue" {
        let bridge_id = light_id.split('-').nth(1).unwrap();
        let light_id = light_id.split('-').nth(2).unwrap();

        let bridge = user.get_huebridge(connection, bridge_id);

        if bridge.is_err() {
            return Err(CustomResponse {
                status: Status::NotFound,
                message: "Bridge not found".to_string(),
            });
        }

        let bridge = bridge.unwrap();

        let light = hue::set_light(&bridge, light_id.to_owned(), state).await;

        if light.is_err() {
            return Err(light.unwrap_err());
        }

        return Ok(());
    }

    Err(CustomResponse {
        status: Status::NotFound,
        message: "Unknown provider".to_string(),
    })
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

async fn get_plug(
    connection: &mut SqlitePooledConnection,
    user: &User,
    plug_id: &String,
) -> Result<NormalizedPlug, CustomResponse> {
    let plug_id = plug_id.to_owned();
    let provider = plug_id.split('-').nth(0).unwrap();

    if provider == "hue" {
        let bridge_id = plug_id.split('-').nth(1).unwrap();
        let plug_id = plug_id.split('-').nth(2).unwrap();

        let bridge = user.get_huebridge(connection, bridge_id);

        if bridge.is_err() {
            return Err(CustomResponse {
                status: Status::NotFound,
                message: "Bridge not found".to_string(),
            });
        }

        let bridge = bridge.unwrap();

        let plug = hue::get_plug(&bridge, &plug_id.to_owned()).await;

        if plug.is_err() {
            return Err(CustomResponse {
                status: Status::NotFound,
                message: "Plug not found".to_string(),
            });
        }

        let plug = plug.unwrap();

        return Ok(plug);
    }

    Err(CustomResponse {
        status: Status::NotFound,
        message: "Unknown provider".to_string(),
    })
}

async fn set_plug_state(
    connection: &mut SqlitePooledConnection,
    user: &User,
    plug_id: &String,
    state: PlugState,
) -> Result<(), CustomResponse> {
    let plug_id = plug_id.to_owned();
    let provider = plug_id.split('-').nth(0).unwrap();

    if provider == "hue" {
        let bridge_id = plug_id.split('-').nth(1).unwrap();
        let plug_id = plug_id.split('-').nth(2).unwrap();

        let bridge = user.get_huebridge(connection, bridge_id);

        if bridge.is_err() {
            return Err(CustomResponse {
                status: Status::NotFound,
                message: "Bridge not found".to_string(),
            });
        }

        let bridge = bridge.unwrap();

        let plug = hue::set_plug(&bridge, plug_id.to_owned(), state).await;

        if plug.is_err() {
            return Err(plug.unwrap_err());
        }

        return Ok(());
    }

    Err(CustomResponse {
        status: Status::NotFound,
        message: "Unknown provider".to_string(),
    })
}

#[openapi]
#[get("/status")]
fn status() -> Json<StatusResponse> {
    let status = StatusResponse {
        status: "OK".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Json(status)
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
#[get("/lights/<light_id>")]
pub async fn light(
    jwt: JWTToken,
    pool: &State<SqlitePool>,
    light_id: String,
) -> Result<Json<NormalizedLight>, CustomResponse> {
    let connection = &mut connection::get_connection(pool).unwrap();

    let user = User::get_user(connection, jwt.user_id);

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Unauthorized".to_string(),
        });
    }

    let user = user.unwrap();

    let response = get_light(connection, &user, &light_id).await;

    if response.is_err() {
        return Err(response.err().unwrap());
    }

    Ok(Json(response.unwrap()))
}

#[openapi(tag = "Main")]
#[put("/lights/<light_id>/state", format = "json", data = "<state>")]
pub async fn set_light(
    jwt: JWTToken,
    pool: &State<SqlitePool>,
    light_id: String,
    state: Json<LightState>,
    queue: &State<Sender<InternalMessage>>,
) -> Result<Json<Value>, CustomResponse> {
    let connection = &mut connection::get_connection(pool).unwrap();

    let user = User::get_user(connection, jwt.user_id);

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Unauthorized".to_string(),
        });
    }

    let user = user.unwrap();

    let response = set_light_state(connection, &user, &light_id, state.into_inner()).await;

    if response.is_err() {
        return Err(response.err().unwrap());
    }

    let response = get_light(connection, &user, &light_id).await;

    if response.is_ok() {
        let _ = queue.send(InternalMessage::light_update(response.unwrap(), jwt));
    }

    Ok(Json(json!({})))
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

#[openapi(tag = "Main")]
#[get("/plugs/<plug_id>")]
pub async fn plug(
    jwt: JWTToken,
    pool: &State<SqlitePool>,
    plug_id: String,
) -> Result<Json<NormalizedPlug>, CustomResponse> {
    let connection = &mut connection::get_connection(pool).unwrap();

    let user = User::get_user(connection, jwt.user_id);

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Unauthorized".to_string(),
        });
    }

    let user = user.unwrap();

    let response = get_plug(connection, &user, &plug_id).await;

    if response.is_err() {
        return Err(response.err().unwrap());
    }

    Ok(Json(response.unwrap()))
}

#[openapi(tag = "Main")]
#[put("/plugs/<plug_id>/state", format = "json", data = "<state>")]
pub async fn set_plug(
    jwt: JWTToken,
    pool: &State<SqlitePool>,
    plug_id: String,
    state: Json<PlugState>,
    queue: &State<Sender<InternalMessage>>,
) -> Result<Json<Value>, CustomResponse> {
    let connection = &mut connection::get_connection(pool).unwrap();

    let user = User::get_user(connection, jwt.user_id);

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Unauthorized".to_string(),
        });
    }

    let user = user.unwrap();

    let response = set_plug_state(connection, &user, &plug_id, state.into_inner()).await;

    if response.is_err() {
        return Err(response.err().unwrap());
    }

    let response = get_plug(connection, &user, &plug_id).await;

    if response.is_ok() {
        let _ = queue.send(InternalMessage::plug_update(response.unwrap(), jwt));
    }

    Ok(Json(json!({})))
}

pub fn routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: status, lights, light, set_light, plugs, plug, set_plug]
}
