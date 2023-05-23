use std::{fs::{create_dir_all}, path::Path};

use okapi::openapi3::OpenApi;
use rocket::{fs::NamedFile, get, http::Status, put, State, data::ByteUnit};
use rocket_okapi::{openapi, openapi_get_routes_spec, settings::OpenApiSettings};

use crate::{
    auth::auth::JWTToken,
    db::{
        connection::{self, SqlitePool},
        models::User,
    },
    repsonses::CustomResponse,
};

use super::assets::{get_picture_path, PictureType};

#[openapi(tag = "User")]
#[get("/profile_pic")]
pub async fn get_profile_pic(
    jwt: JWTToken,
    pool: &State<SqlitePool>,
) -> Result<NamedFile, CustomResponse> {
    let connection = &mut connection::get_connection(pool).unwrap();

    let user = User::get_user(connection, jwt.user_id);

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Unauthorized".to_string(),
        });
    }

    let user = user.unwrap();

    let picture = get_picture_path(PictureType::ProfilePic { user_id: user.id });

    if !Path::new(&picture).exists() {
        return Err(CustomResponse {
            status: Status::NotFound,
            message: "Not Found".to_string(),
        });
    }

    match NamedFile::open(picture).await.ok() {
        Some(file) => Ok(file),
        None => Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Internal Server Error".to_string(),
        }),
    }
}

#[openapi(tag = "User")]
#[put("/profile_pic", data = "<data>")]
pub async fn put_profile_pic(
    jwt: JWTToken,
    pool: &State<SqlitePool>,
    data: rocket::Data<'_>,
) -> Result<Status, CustomResponse> {
    let connection = &mut connection::get_connection(pool).unwrap();

    let user = User::get_user(connection, jwt.user_id);

    if user.is_err() {
        return Err(CustomResponse {
            status: Status::Unauthorized,
            message: "Unauthorized".to_string(),
        });
    }

    let user = user.unwrap();

    let picture = get_picture_path(PictureType::ProfilePic { user_id: user.id });

    if !Path::new(&picture).parent().unwrap().exists() {
        create_dir_all(Path::new(&picture).parent().unwrap()).unwrap();
    }

    let result = data.open(ByteUnit::Megabyte(15)).into_file(Path::new(&picture)).await;

    if result.is_err() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Internal Server Error".to_string(),
        });
    }

    if !result.unwrap().is_complete() {
        return Err(CustomResponse {
            status: Status::InternalServerError,
            message: "Error Writing File".to_string(),
        });
    }

    Ok(Status::Ok)
}

pub fn routes(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: get_profile_pic, put_profile_pic]
}
