use okapi::openapi3::OpenApi;
use rocket::{get, http::Status, post, serde::json::Json, State};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};
use schemars::JsonSchema;

use crate::{
    db::{
        connection::{self, SqlitePool, SqlitePooledConnection},
        models::{
            HueBridge, NewHueBridge, NewUser, NewWledItem, UpdateUserSettings, User, WledItem,
        },
    },
    repsonses::CustomResponse,
};

use super::auth::{hash_password, verify_password, JWTToken};

#[derive(serde::Deserialize, JsonSchema)]
struct HueBridgeRequest {
    host: String,
    user: String,
}

#[derive(serde::Deserialize, JsonSchema)]
struct WledItemRequest {
    name: String,
    ip: String,
}

#[derive(serde::Deserialize, JsonSchema)]
struct UserSettingsRequest {
    hue_bridges: Vec<HueBridgeRequest>,
    wled_ips: Vec<WledItemRequest>,
}

#[derive(serde::Deserialize, JsonSchema)]
struct SignupRequest {
    username: String,
    email: String,
    password: String,
    settings: Option<UserSettingsRequest>,
}

#[derive(serde::Serialize, JsonSchema)]
struct SignupResponse {
    access_token: String,
    token_type: String,
}

fn connection_from_pool(pool: &State<SqlitePool>) -> SqlitePooledConnection {
    connection::get_connection(&pool).unwrap()
}

#[openapi(tag = "Auth")]
#[post("/signup", format = "json", data = "<signup_request>")]
fn signup(
    signup_request: Json<SignupRequest>,
    dbpool: &State<SqlitePool>,
) -> Result<Json<SignupResponse>, CustomResponse> {
    let connection = &mut connection_from_pool(dbpool);

    let user_by_username = User::get_user_by_username(connection, &signup_request.username);

    if user_by_username.is_ok() {
        return Err(CustomResponse {
            status: Status::Conflict,
            message: "Username already exists".to_string(),
        });
    }

    let user_by_email = User::get_user_by_mail(connection, &signup_request.email);

    if user_by_email.is_ok() {
        return Err(CustomResponse {
            status: Status::Conflict,
            message: "Email already exists".to_string(),
        });
    }

    let user = User::create_user(
        connection,
        &NewUser {
            username: &signup_request.username,
            email: &signup_request.email,
            hashed_password: &hash_password(&signup_request.password),
        },
    );

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Error creating user".to_string(),
        });
    }

    let user = user.unwrap();

    if let Some(user_settings_obj) = &signup_request.settings {
        let mut user_settings = user.get_usersettings(connection).unwrap();

        if user_settings_obj.hue_bridges.len() > 0 {
            let result = user_settings.update(
                connection,
                &UpdateUserSettings {
                    hue_index: Some(&(user_settings.hue_index + 1)),
                    user_id: None,
                },
            );

            if result.is_err() {
                println!("Error updating user settings: {:?}", result);
                return Err(CustomResponse {
                    status: Status::InternalServerError,
                    message: "Error updating user settings".to_string(),
                });
            }

            user_settings = result.unwrap();

            for hue_bridge in user_settings_obj.hue_bridges.iter() {
                let hue_bridge = HueBridge::create_huebridge(
                    connection,
                    &NewHueBridge {
                        id: &user_settings.hue_index.to_string(),
                        user: &hue_bridge.user,
                        ip: &hue_bridge.host,
                        user_settings_id: &user_settings.id,
                    },
                );

                if hue_bridge.is_err() {
                    println!("Error creating hue bridge: {:?}", hue_bridge);
                    return Err(CustomResponse {
                        status: Status::InternalServerError,
                        message: "Error creating hue bridge".to_string(),
                    });
                }
            }
        }

        if user_settings_obj.wled_ips.len() > 0 {
            for wled_ip in user_settings_obj.wled_ips.iter() {
                let wled_item = WledItem::create_wleditem(
                    connection,
                    &NewWledItem {
                        name: &wled_ip.name,
                        ip: &wled_ip.ip,
                        user_settings_id: &user_settings.id,
                    },
                );

                if wled_item.is_err() {
                    println!("Error creating wled item: {:?}", wled_item);
                    return Err(CustomResponse {
                        status: Status::InternalServerError,
                        message: "Error creating wled item".to_string(),
                    });
                }
            }
        }
    }

    Ok(Json(SignupResponse {
        access_token: user.generate_token(),
        token_type: "bearer".to_string(),
    }))
}

#[derive(serde::Deserialize, JsonSchema)]
struct LoginRequest {
    email: String,
    password: String,
}

#[openapi(tag = "Auth")]
#[post("/login", format = "json", data = "<login_request>")]
fn login(
    login_request: Json<LoginRequest>,
    dbpool: &State<SqlitePool>,
) -> Result<Json<SignupResponse>, CustomResponse> {
    let connection = &mut connection_from_pool(dbpool);

    let user = User::get_user_by_mail(connection, &login_request.email);

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Invalid email or password".to_string(),
        });
    }

    let user = user.unwrap();

    if !verify_password(&login_request.password, &user.hashed_password) {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Invalid email or password".to_string(),
        });
    }

    Ok(Json(SignupResponse {
        access_token: user.generate_token(),
        token_type: "bearer".to_string(),
    }))
}

#[derive(serde::Serialize, JsonSchema)]
struct MeResponse {
    email: String,
}

#[openapi(tag = "Auth")]
#[get("/refresh")]
fn refresh(
    jwt: JWTToken,
    db_pool: &State<SqlitePool>,
) -> Result<Json<SignupResponse>, CustomResponse> {
    let connection = &mut connection_from_pool(db_pool);

    let user = User::get_user(connection, jwt.user_id);

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Invalid token".to_string(),
        });
    }

    let user = user.unwrap();

    Ok(Json(SignupResponse {
        access_token: user.generate_token(),
        token_type: "bearer".to_string(),
    }))
}

#[openapi(tag = "Auth")]
#[get("/me")]
fn me(jwt: JWTToken, db_pool: &State<SqlitePool>) -> Result<Json<MeResponse>, CustomResponse> {
    let connection = &mut connection_from_pool(db_pool);

    let user = User::get_user(connection, jwt.user_id);

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Invalid token".to_string(),
        });
    }

    let user = user.unwrap();

    Ok(Json(MeResponse {
        email: user.email.to_string(),
    }))
}

pub fn routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: signup, login, refresh, me]
}
