use rocket::{
    get,
    http::Status,
    put,
    serde::{self, json::Json},
    State,
};
use rocket_okapi::{openapi, openapi_get_routes};
use schemars::{
    JsonSchema,
    _serde_json::{self, Value},
};

use crate::db::{
    connection::{self, SqlitePool, SqlitePooledConnection},
    models::{HueBridge, NewHueBridge, UpdateHueBridge, UpdateUserSettings, User},
};

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

async fn post_hue_json(hue_bridge: &HueBridge, path: String, body: String) -> String {
    let res = client()
        .post(&format!("http://{}/api/{}", hue_bridge.ip, path))
        .body(body)
        .send()
        .await
        .unwrap()
        .text()
        .await;

    res.unwrap()
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

#[derive(serde::Serialize, JsonSchema)]
struct ConfigResponse {
    id: String,
}

#[derive(serde::Deserialize, JsonSchema)]
struct ConfigRequest {
    host: Option<String>,
    user: Option<String>,
}
// BIG TODO: implement jwt auth
#[openapi]
#[put("/config/add", format = "json", data = "<config_json>")]
async fn add_config(
    _dbpool: &State<SqlitePool>,
    config_json: Json<ConfigRequest>,
) -> Result<Json<ConfigResponse>, Status> {
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

    let user = User::get_user(connection, 2).unwrap();

    let mut usersettings = user.get_usersettings(connection).unwrap();

    let hue_bridges = usersettings.get_huebridges(connection).unwrap();

    for hue_bridge in hue_bridges {
        if hue_bridge.ip == config_ip {
            return Err(Status::Conflict);
        }
    }

    usersettings = usersettings
        .update(
            connection,
            &UpdateUserSettings {
                hue_index: Some(&(usersettings.hue_index + 1)),
                user_id: None,
            },
        )
        .unwrap();

    let hue_bridge = HueBridge::create_huebridge(
        connection,
        &NewHueBridge {
            id: &usersettings.hue_index.to_string(),
            ip: config_ip,
            user: config_user,
            user_settings_id: &usersettings.id,
        },
    )
    .unwrap();

    Ok(Json(ConfigResponse {
        id: hue_bridge.id.to_owned(),
    }))
}

#[derive(serde::Serialize, JsonSchema)]
struct InitResponse {
    username: String,
}

#[openapi]
#[get("/init/<bridge_id>")]
async fn init(
    _dbpool: &State<SqlitePool>,
    bridge_id: String,
) -> Result<Json<InitResponse>, Status> {
    let connection = &mut connection_from_pool(_dbpool);

    let hue_bridge = &mut HueBridge::get_huebridge_by_bridge_id(connection, &bridge_id).unwrap();

    let user_request = post_hue_json(
        hue_bridge,
        "".to_string(),
        "{\"devicetype\":\"hue#home api rust\"}".to_string(),
    )
    .await;

    let json_value: Value = _serde_json::de::from_str(&user_request).unwrap();

    let json = json_value.clone()[0].clone();
    let error = json["error"].clone();
    if !error.is_null() {
        if error["type"] == 101 {
            return Err(Status::BadRequest);
        }
    }

    let username = json["success"]["username"].as_str().unwrap();

    hue_bridge
        .update(
            connection,
            &UpdateHueBridge {
                id: None,
                ip: None,
                user: Some(username),
                user_settings_id: None,
            },
        )
        .unwrap();

    Ok(Json(InitResponse {
        username: username.to_owned(),
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    openapi_get_routes![init, add_config]
}
