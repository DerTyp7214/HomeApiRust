use ::serde::{Deserialize, Serialize};
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
    _serde_json::{self, json, Value},
};

use crate::{
    auth::auth::JWTToken,
    db::{
        connection::{self, SqlitePool, SqlitePooledConnection},
        models::{HueBridge, NewHueBridge, UpdateHueBridge, UpdateUserSettings, User},
    },
    repsonses::CustomResponse,
    utils::color::{hsb_to_hsv, hsv_to_hsb, hsv_to_rgb, rgb_to_hsv},
};

use super::main::{LightState, NormalizedColor, NormalizedLight, NormalizedPlug, PlugState};
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HueScene {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub group: String,
    pub lights: Vec<String>,
    pub owner: String,
    pub recycle: bool,
    pub locked: bool,
    pub appdata: HueAppData,
    pub picture: String,
    pub lastupdated: String,
    pub version: i32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HueAppData {
    pub version: i32,
    pub data: Value,
}

async fn get_hue_json(hue_bridge: &HueBridge, mut path: String) -> Result<String, CustomResponse> {
    if path.starts_with('/') {
        path.remove(0);
    }
    let res = client()
        .get(&format!(
            "http://{}/api/{}/{}",
            hue_bridge.ip, hue_bridge.user, path
        ))
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

async fn post_hue_json(
    hue_bridge: &HueBridge,
    mut path: String,
    body: String,
) -> Result<String, CustomResponse> {
    if path.starts_with('/') {
        path.remove(0);
    }
    let res = client()
        .post(&format!(
            "http://{}/api/{}/{}",
            hue_bridge.ip, hue_bridge.user, path
        ))
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

async fn put_hue_json(
    hue_bridge: &HueBridge,
    mut path: String,
    body: String,
) -> Result<String, CustomResponse> {
    if path.starts_with('/') {
        path.remove(0);
    }
    let res = client()
        .put(&format!(
            "http://{}/api/{}/{}",
            hue_bridge.ip, hue_bridge.user, path
        ))
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

#[derive(Serialize, Deserialize, JsonSchema)]
struct HueLightState {
    on: Option<bool>,
    sat: Option<u8>,
    bri: Option<u8>,
    hue: Option<u16>,
}

pub async fn set_light(
    hue_bridge: &HueBridge,
    light_id: String,
    light_state: LightState,
) -> Result<Status, CustomResponse> {
    if hue_bridge.user.is_empty() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Hue Bridge not configured".to_owned(),
        });
    }

    let brigthness = light_state.brigthness;
    let color = light_state.color;

    if light_state.on.is_none() && brigthness.is_none() && color.is_none() {
        return Ok(Status::Ok);
    }

    let mut hsb: Option<(u16, u8, u8)> = None;

    if color.is_some() {
        let color = color.unwrap();
        if color.len() > 0 {
            let hsv = rgb_to_hsv(
                color.get(0).unwrap().0,
                color.get(0).unwrap().1,
                color.get(0).unwrap().2,
            );

            hsb = Some(hsv_to_hsb(hsv.0, hsv.1, hsv.2));
        }
    }

    let mut body = HueLightState {
        on: light_state.on,
        hue: None,
        sat: None,
        bri: None,
    };

    if hsb.is_some() {
        body.hue = Some(hsb.unwrap().0);
        body.sat = Some(hsb.unwrap().1);
        body.bri = Some(hsb.unwrap().2);
    }

    let response = put_hue_json(
        hue_bridge,
        format!("lights/{}/state", light_id),
        _serde_json::ser::to_string(&body).unwrap(),
    )
    .await;

    if response.is_err() {
        return Err(response.unwrap_err());
    }

    Ok(Status::Ok)
}

pub async fn get_lights(hue_bridge: &HueBridge) -> Vec<NormalizedLight> {
    if hue_bridge.user.is_empty() {
        return Vec::new();
    }
    let response = get_hue_json(hue_bridge, "lights".to_owned()).await;

    if response.is_err() {
        return Vec::new();
    }

    let response = response.unwrap();

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

pub async fn get_light(
    hue_bridge: &HueBridge,
    light_id: &String,
) -> Result<NormalizedLight, CustomResponse> {
    let light_id = light_id.to_owned();
    if hue_bridge.user.is_empty() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Hue Bridge not configured".to_owned(),
        });
    }

    let response = get_hue_json(hue_bridge, format!("lights/{}", light_id.to_owned())).await;

    if response.is_err() {
        return Err(response.unwrap_err());
    }

    let response = response.unwrap();

    let json: Value = _serde_json::de::from_str(&response).unwrap();

    let json = json.as_object();

    if json.is_none() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Failed to parse Hue Bridge response".to_owned(),
        });
    }

    let json = json.unwrap();

    let light = json.get("state").unwrap().as_object().unwrap();

    if !light.contains_key("colormode") {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Light is not a color light".to_owned(),
        });
    }

    if !light.contains_key("reachable") || !light.get("reachable").to_bool() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Light is not reachable".to_owned(),
        });
    }

    let hsv = hsb_to_hsv(
        light.get("hue").to_f64() as f32,
        light.get("sat").to_f64() as f32,
        light.get("bri").to_f64() as f32,
    );

    let rgb = hsv_to_rgb(hsv.0, hsv.1, hsv.2);

    Ok(NormalizedLight {
        id: format!("hue-{}-{}", hue_bridge.id, light_id),
        name: json.get("name").to_string(),
        on: light.get("on").to_bool(),
        brightness: (light.get("bri").to_f64() / 255.0) as f32,
        color: vec![NormalizedColor(rgb.0, rgb.1, rgb.2)],
        reachable: light.get("reachable").to_bool(),
        type_: json.get("type").to_string(),
        model: json.get("modelid").to_string(),
        manufacturer: json.get("manufacturername").to_string(),
        uniqueid: json.get("uniqueid").to_string(),
        swversion: json.get("swversion").to_string(),
        productid: Some(json.get("productid").to_string()),
    })
}

pub async fn set_plug(
    hue_bridge: &HueBridge,
    plug_id: String,
    plug_state: PlugState,
) -> Result<Status, CustomResponse> {
    if plug_state.on.is_none() {
        return Ok(Status::Ok);
    }

    if hue_bridge.user.is_empty() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Hue Bridge not configured".to_owned(),
        });
    }

    let response = put_hue_json(
        hue_bridge,
        format!("lights/{}/state", plug_id),
        _serde_json::ser::to_string(&plug_state).unwrap(),
    )
    .await;

    if response.is_err() {
        return Err(response.unwrap_err());
    }

    Ok(Status::Ok)
}

pub async fn get_plugs(hue_bridge: &HueBridge) -> Vec<NormalizedPlug> {
    if hue_bridge.user.is_empty() {
        return Vec::new();
    }
    let response = get_hue_json(hue_bridge, "lights".to_owned()).await;

    if response.is_err() {
        return Vec::new();
    }

    let response = response.unwrap();

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

pub async fn get_plug(
    hue_bridge: &HueBridge,
    plug_id: &String,
) -> Result<NormalizedPlug, CustomResponse> {
    let plug_id = plug_id.to_owned();
    if hue_bridge.user.is_empty() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Hue Bridge not configured".to_owned(),
        });
    }

    let response = get_hue_json(hue_bridge, format!("lights/{}", plug_id)).await;

    if response.is_err() {
        return Err(response.unwrap_err());
    }

    let response = response.unwrap();

    let json: Value = _serde_json::de::from_str(&response).unwrap();

    let json = json.as_object();

    if json.is_none() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Failed to parse Hue Bridge response".to_owned(),
        });
    }

    let json = json.unwrap();

    let light = json.get("state").unwrap().as_object().unwrap();

    if !json
        .get("config")
        .unwrap()
        .as_object()
        .unwrap()
        .get("archetype")
        .to_string()
        .eq("plug")
    {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Light is not a plug".to_owned(),
        });
    }

    if !json.contains_key("reachable") || !json.get("reachable").to_bool() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Plug is not reachable".to_owned(),
        });
    }

    Ok(NormalizedPlug {
        id: format!("hue-{}-{}", hue_bridge.id, plug_id),
        name: json.get("name").to_string(),
        on: light.get("on").to_bool(),
        reachable: light.get("reachable").to_bool(),
        type_: json.get("type").to_string(),
        model: json.get("modelid").to_string(),
        manufacturer: json.get("manufacturername").to_string(),
        uniqueid: json.get("uniqueid").to_string(),
        swversion: json.get("swversion").to_string(),
        productid: Some(json.get("productid").to_string()),
    })
}

async fn __get_scenes__(hue_bridge: &HueBridge) -> Result<Vec<HueScene>, CustomResponse> {
    if hue_bridge.user.is_empty() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Hue Bridge not configured".to_owned(),
        });
    }

    let response = get_hue_json(hue_bridge, "scenes".to_owned()).await;

    if response.is_err() {
        return Err(response.unwrap_err());
    }

    let response = response.unwrap();

    let json: Value = _serde_json::de::from_str(&response).unwrap();

    let json = json.as_object().unwrap();

    let mut scenes = Vec::new();

    for (id, scene_json) in json.iter() {
        let mut scene_json = scene_json.as_object().unwrap().to_owned();
        scene_json.insert("id".to_owned(), id.to_owned().into());

        let scene: HueScene = _serde_json::from_value(Value::Object(scene_json)).unwrap();
        scenes.push(scene);
    }

    Ok(scenes)
}

async fn __set_scene__(
    hue_bridge: &HueBridge,
    scene_id: &String,
    group_id: &String,
) -> Result<String, CustomResponse> {
    if hue_bridge.user.is_empty() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Hue Bridge not configured".to_owned(),
        });
    }

    let response = put_hue_json(
        hue_bridge,
        format!("groups/{}/action", group_id),
        format!("{{\"scene\": \"{}\"}}", scene_id),
    )
    .await;

    if response.is_err() {
        return Err(response.unwrap_err());
    }

    let response = response.unwrap();

    let json = _serde_json::from_str::<Value>(&response).unwrap();

    if json.is_array() {
        let json = json.as_array().unwrap();

        for json in json.iter() {
            let json = json.as_object().unwrap();

            if json.contains_key("error") {
                let error: Option<&_serde_json::Map<String, Value>> =
                    json.get("error").unwrap().as_object();

                if error.is_none() {
                    return Err(CustomResponse {
                        status: Status::InternalServerError,
                        message: json.get("error").unwrap().as_str().unwrap().to_owned(),
                    });
                }

                let error = error.unwrap();

                if error.contains_key("description") {
                    return Err(CustomResponse {
                        status: Status::InternalServerError,
                        message: error
                            .get("description")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_owned(),
                    });
                }
            }

            if json.contains_key("success") {
                let json = json.get("success").unwrap().as_object().unwrap();

                return Ok(json.values().next().unwrap().as_str().unwrap().to_owned());
            }
        }
    }

    Ok(response)
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
    jwt: JWTToken,
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

    let user = User::get_user(connection, jwt.user_id);

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
            id: &usersettings.hue_index.to_owned().to_string(),
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
    jwt: JWTToken,
    _dbpool: &State<SqlitePool>,
) -> Result<Json<Vec<HueBridge>>, CustomResponse> {
    let connection = &mut connection_from_pool(_dbpool);

    let hue_bridges = HueBridge::get_huebridges_by_user_id(connection, jwt.user_id);

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
#[get("/scenes/<bridge_id>")]
async fn get_scenes(
    jwt: JWTToken,
    _dbpool: &State<SqlitePool>,
    bridge_id: String,
) -> Result<Json<Vec<HueScene>>, CustomResponse> {
    let connection = &mut connection_from_pool(_dbpool);

    let hue_bridge = HueBridge::get_huebridge_by_bridge_id(connection, jwt.user_id, &bridge_id);

    println!("{:?}", hue_bridge);
    print!("{}", jwt.user_id);
    print!("{}", bridge_id);

    if hue_bridge.is_err() {
        return Err(CustomResponse {
            status: Status::NotFound,
            message: "Bridge not found".to_string(),
        });
    }

    let hue_bridge = hue_bridge.unwrap();

    let hue_scenes = __get_scenes__(&hue_bridge).await;

    if hue_scenes.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Could not get hue scenes".to_string(),
        });
    }

    let hue_scenes = hue_scenes.unwrap();

    Ok(Json(hue_scenes))
}

#[openapi(tag = "Hue")]
#[put("/scenes/<bridge_id>/<group_id>/<scene_id>")]
async fn set_scene(
    jwt: JWTToken,
    _dbpool: &State<SqlitePool>,
    bridge_id: String,
    scene_id: String,
    group_id: String,
) -> Result<Json<Value>, CustomResponse> {
    let connection = &mut connection_from_pool(_dbpool);

    let hue_bridge = HueBridge::get_huebridge_by_bridge_id(connection, jwt.user_id, &bridge_id);

    if hue_bridge.is_err() {
        return Err(CustomResponse {
            status: Status::NotFound,
            message: "Bridge not found".to_string(),
        });
    }

    let hue_bridge = hue_bridge.unwrap();

    let response = __set_scene__(&hue_bridge, &scene_id, &group_id).await;

    if response.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Could not set hue scene".to_string(),
        });
    }

    Ok(Json(json!({"success": response.unwrap()})))
}

#[openapi(tag = "Hue")]
#[delete("/config/<bridge_id>")]
async fn delete_bridge(
    jwt: JWTToken,
    _dbpool: &State<SqlitePool>,
    bridge_id: String,
) -> Result<Json<Value>, CustomResponse> {
    let connection = &mut connection_from_pool(_dbpool);

    let hue_bridge = HueBridge::get_huebridge_by_bridge_id(connection, jwt.user_id, &bridge_id);

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

    Ok(Json(json!({})))
}

#[openapi(tag = "Hue")]
#[get("/init/<bridge_id>")]
async fn init(
    jwt: JWTToken,
    _dbpool: &State<SqlitePool>,
    bridge_id: String,
) -> Result<Json<InitResponse>, CustomResponse> {
    let connection = &mut connection_from_pool(_dbpool);

    let hue_bridge = HueBridge::get_huebridge_by_bridge_id(connection, jwt.user_id, &bridge_id);

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
    openapi_get_routes_spec![settings: init, add_config, get_bridges, delete_bridge, get_scenes, set_scene]
}
