use rocket::{
    get,
    http::Status,
    put,
    serde::{self, json::Json},
};
use rocket_okapi::{openapi, openapi_get_routes};
use schemars::{
    JsonSchema,
    _serde_json::{self, Value},
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

async fn get_hue_json(path: String) -> String {
    let res = client()
        .get(&format!("http://192.168.178.211/api/{}", path))
        .send()
        .await
        .ok()
        .unwrap()
        .text()
        .await;

    res.unwrap()
}

async fn post_hue_json(path: String, body: String) -> String {
    let res = client()
        .post(&format!("http://192.168.178.211/api/{}", path))
        .body(body)
        .send()
        .await
        .unwrap()
        .text()
        .await;

    res.unwrap()
}

async fn put_hue_json(path: String, body: String) -> String {
    let res = client()
        .put(&format!("http://192.168.178.211/api/{}", path))
        .body(body)
        .send()
        .await
        .unwrap()
        .text()
        .await;

    res.unwrap()
}

#[derive(serde::Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
struct ConfigResponse {
    id: String,
}

#[openapi]
#[put("/config/add")]
async fn add_config() -> Result<Json<ConfigResponse>, Status> {
    
    Ok(Json(ConfigResponse {
        id: "1".to_string(),
    }))
}

#[derive(serde::Serialize, JsonSchema)]
#[serde(crate = "rocket::serde")]
struct InitResponse {
    username: String,
}

#[openapi]
#[get("/init/<bridge_id>")]
async fn init(bridge_id: String) -> Result<Json<InitResponse>, Status> {
    let user_request = post_hue_json(
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

    Ok(Json(InitResponse {
        username: json["success"]["username"].as_str().unwrap().to_owned(),
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    openapi_get_routes![init, add_config]
}
