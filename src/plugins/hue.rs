use okapi::openapi3::OpenApi;
use rocket::{
    delete, get,
    http::Status,
    put,
    serde::{self, json::Json},
    State,
};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use schemars::{
    JsonSchema,
    _serde_json::{self, Value},
};

use crate::{
    auth::auth::JWTToken,
    db::{
        connection::{self, SqlitePool, SqlitePooledConnection},
        models::{HueBridge, NewHueBridge, UpdateHueBridge, UpdateUserSettings, User},
    },
    repsonses::CustomResponse,
    utils::color::{hsb_to_hsv, hsv_to_rgb},
};

use super::main::{NormalizedColor, NormalizedLight, NormalizedPlug};
use crate::utils::extensions::ValueExt;

static mut STATIC_CLIENT: Option<reqwest::Client> = None;

fn client() -> reqwest::Client {
    unsafe {
        if STATIC_CLIENT.is_none() {
            STATIC_CLIENT = Some(reqwest::Client::new());
        }

        STATIC_CLIENT.clone().unwrap()
    }
}

fn connection_from_pool(pool: &State<SqlitePool>) -> SqlitePooledConnection {
    connection::get_connection(&pool).unwrap()
}

async fn get_hue_json(hue_bridge: &HueBridge, path: String) -> String {
    let res = client()
        .get(&format!("http://{}/api/{}", hue_bridge.ip, path))
        .send()
        .await
        .ok()
        .unwrap()
        .text()
        .await;

    res.unwrap()
}

async fn post_hue_json(
    hue_bridge: &HueBridge,
    path: String,
    body: String,
) -> Result<String, CustomResponse> {
    let res = client()
        .post(&format!("http://{}/api/{}", hue_bridge.ip, path))
        .body(body)
        .send()
        .await;

    if res.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Failed to connect to Hue Bridge".to_owned(),
        });
    }

    let res = res.unwrap().text().await;

    if res.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Failed to connect to Hue Bridge".to_owned(),
        });
    }

    Ok(res.unwrap())
}

async fn put_hue_json(hue_bridge: &HueBridge, path: String, body: String) -> String {
    let res = client()
        .put(&format!("http://{}/api/{}", hue_bridge.ip, path))
        .body(body)
        .send()
        .await
        .unwrap()
        .text()
        .await;

    res.unwrap()
}

pub async fn get_lights(hue_bridge: &HueBridge) -> Vec<NormalizedLight> {
    if hue_bridge.user.is_empty() {
        return Vec::new();
    }
    let response = get_hue_json(hue_bridge, format!("{}/lights", hue_bridge.user)).await;
    let json: Value = _serde_json::de::from_str(&response).unwrap();

    let json = json.as_object().unwrap();

    let mut lights = Vec::new();

    for (id, light_json) in json.iter() {
        let light = light_json.as_object().unwrap();
        let state = light.get("state").unwrap().as_object().unwrap();

        if state.contains_key("colormode") {
            let hsv = hsb_to_hsv(
                state.get("hue").to_f64() as f32,
                state.get("sat").to_f64() as f32,
                state.get("bri").to_f64() as f32,
            );
            let rgb = hsv_to_rgb(hsv.0, hsv.1, hsv.2);

            lights.push(NormalizedLight {
                id: format!("hue-{}-{}", hue_bridge.id, id),
                name: light.get("name").to_string(),
                on: state.get("on").to_bool(),
                brightness: (state.get("bri").to_f64() / 255.0) as f32,
                color: vec![NormalizedColor(rgb.0, rgb.1, rgb.2)],
                reachable: state.get("reachable").to_bool(),
                type_: light.get("type").to_string(),
                model: light.get("modelid").to_string(),
                manufacturer: light.get("manufacturername").to_string(),
                uniqueid: light.get("uniqueid").to_string(),
                swversion: light.get("swversion").to_string(),
                productid: Some(light.get("productid").to_string()),
            });
        }
    }

    lights
}

pub async fn get_plugs(hue_bridge: &HueBridge) -> Vec<NormalizedPlug> {
    if hue_bridge.user.is_empty() {
        return Vec::new();
    }
    let response = get_hue_json(hue_bridge, format!("{}/lights", hue_bridge.user)).await;
    let json: Value = _serde_json::de::from_str(&response).unwrap();

    let json = json.as_object().unwrap();

    let mut plugs = Vec::new();

    for (id, light_json) in json.iter() {
        let light = light_json.as_object().unwrap();
        let state = light.get("state").unwrap().as_object().unwrap();

        if light
            .get("config")
            .unwrap()
            .as_object()
            .unwrap()
            .get("archetype")
            .to_string()
            .eq("plug")
        {
            plugs.push(NormalizedPlug {
                id: format!("hue-{}-{}", hue_bridge.id, id),
                name: light.get("name").to_string(),
                on: state.get("on").to_bool(),
                reachable: state.get("reachable").to_bool(),
                type_: light.get("type").to_string(),
                model: light.get("modelid").to_string(),
                manufacturer: light.get("manufacturername").to_string(),
                uniqueid: light.get("uniqueid").to_string(),
                swversion: light.get("swversion").to_string(),
                productid: Some(light.get("productid").to_string()),
            });
        }
    }

    plugs
}

#[derive(serde::Serialize, JsonSchema)]
struct ConfigResponse {
    id: String,
}

#[derive(serde::Deserialize, JsonSchema)]
struct ConfigRequest {
    host: Option<String>,
    user: Option<String>,
}

#[openapi(tag = "Hue")]
#[put("/config/add", format = "json", data = "<config_json>")]
async fn add_config(
    jtw: JWTToken,
    _dbpool: &State<SqlitePool>,
    config_json: Json<ConfigRequest>,
) -> Result<Json<ConfigResponse>, CustomResponse> {
    let connection = &mut connection::get_connection(_dbpool).unwrap();

    let config = config_json.into_inner();

    let config_ip = if let Some(host) = &config.host {
        host
    } else {
        ""
    };

    let config_user = if let Some(user) = &config.user {
        user
    } else {
        ""
    };

    let user = User::get_user(connection, jtw.user_id);

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Unauthorized".to_string(),
        });
    }

    let user = user.unwrap();

    let usersettings = user.get_usersettings(connection);

    if usersettings.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Could not get usersettings".to_string(),
        });
    }

    let usersettings = usersettings.unwrap();

    let hue_bridges = usersettings.get_huebridges(connection);

    if hue_bridges.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Could not get hue bridges".to_string(),
        });
    }

    let hue_bridges = hue_bridges.unwrap();

    for hue_bridge in hue_bridges {
        if hue_bridge.ip == config_ip {
            return Err(CustomResponse {
                status: Status::Conflict,
                message: "Bridge already exists".to_string(),
            });
        }
    }

    let usersettings = usersettings.update(
        connection,
        &UpdateUserSettings {
            hue_index: Some(&(usersettings.hue_index + 1)),
            user_id: None,
        },
    );

    if usersettings.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Could not update usersettings".to_string(),
        });
    }

    let usersettings = usersettings.unwrap();

    let hue_bridge = HueBridge::create_huebridge(
        connection,
        &NewHueBridge {
            id: &usersettings.hue_index.to_string(),
            ip: config_ip,
            user: config_user,
            user_settings_id: &usersettings.id,
        },
    );

    if hue_bridge.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Could not create hue bridge".to_string(),
        });
    }

    let hue_bridge = hue_bridge.unwrap();

    Ok(Json(ConfigResponse {
        id: hue_bridge.id.to_owned(),
    }))
}

#[derive(serde::Serialize, JsonSchema)]
struct InitResponse {
    username: String,
}

#[openapi(tag = "Hue")]
#[get("/bridges")]
async fn get_bridges(
    jtw: JWTToken,
    _dbpool: &State<SqlitePool>,
) -> Result<Json<Vec<HueBridge>>, CustomResponse> {
    let connection = &mut connection_from_pool(_dbpool);

    let hue_bridges = HueBridge::get_huebridges_by_user_id(connection, jtw.user_id);

    if hue_bridges.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Could not get hue bridges".to_string(),
        });
    }

    let hue_bridges = hue_bridges.unwrap();

    Ok(Json(hue_bridges))
}

#[openapi(tag = "Hue")]
#[delete("/config/<bridge_id>")]
async fn delete_bridge(
    jtw: JWTToken,
    _dbpool: &State<SqlitePool>,
    bridge_id: String,
) -> Result<Json<()>, CustomResponse> {
    let connection = &mut connection_from_pool(_dbpool);

    let hue_bridge = HueBridge::get_huebridge_by_bridge_id(connection, jtw.user_id, &bridge_id);

    if hue_bridge.is_err() {
        return Err(CustomResponse {
            status: Status::NotFound,
            message: "Bridge not found".to_string(),
        });
    }

    let hue_bridge = hue_bridge.unwrap();

    let hue_bridge = hue_bridge.delete(connection);

    if hue_bridge.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Could not delete hue bridge".to_string(),
        });
    }

    Ok(Json(()))
}

#[openapi(tag = "Hue")]
#[get("/init/<bridge_id>")]
async fn init(
    jtw: JWTToken,
    _dbpool: &State<SqlitePool>,
    bridge_id: String,
) -> Result<Json<InitResponse>, CustomResponse> {
    let connection = &mut connection_from_pool(_dbpool);

    let hue_bridge = HueBridge::get_huebridge_by_bridge_id(connection, jtw.user_id, &bridge_id);

    if hue_bridge.is_err() {
        return Err(CustomResponse {
            status: Status::NotFound,
            message: "Bridge not found".to_string(),
        });
    }

    let hue_bridge = &mut hue_bridge.unwrap();

    let user_request = post_hue_json(
        hue_bridge,
        "".to_string(),
        "{\"devicetype\":\"hue#home api rust\"}".to_string(),
    )
    .await;

    if user_request.is_err() {
        return Err(user_request.unwrap_err());
    }

    let user_request = user_request.unwrap();

    let json_value: Value = _serde_json::de::from_str(&user_request).unwrap();

    let json = json_value.clone()[0].clone();
    let error = json["error"].clone();
    if !error.is_null() {
        if error["type"] == 101 {
            return Err(CustomResponse {
                status: Status::Unauthorized,
                message: "Link button not pressed".to_string(),
            });
        }
    }

    let username = json["success"]["username"].as_str().unwrap();

    let hue_bridge = hue_bridge.update(
        connection,
        &UpdateHueBridge {
            id: None,
            ip: None,
            user: Some(username),
            user_settings_id: None,
        },
    );

    if hue_bridge.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Could not update hue bridge".to_string(),
        });
    }

    Ok(Json(InitResponse {
        username: username.to_owned(),
    }))
}

pub fn routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: init, add_config, get_bridges, delete_bridge]
}
