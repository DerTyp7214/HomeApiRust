use okapi::openapi3::Responses;
use rocket::{http::Status, response::Responder};
use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};
use rocket_okapi::{gen::OpenApiGenerator, response::OpenApiResponderInner};
use schemars::{JsonSchema, Map, _serde_json::json};

#[derive(Debug)]
pub struct CustomResponse {
    pub status: Status,
    pub message: String,
}

#[derive(JsonSchema)]
struct Schema {
    #[allow(dead_code)]
    message: String,
}

fn gen_response(status: Status) -> OpenApiReponse {
    let mut response = OpenApiReponse::default();
    response.description = status.reason_lossy().to_owned();
    response.content = vec![(
        "application/json".to_owned(),
        rocket_okapi::okapi::openapi3::MediaType {
            schema: Some(schemars::schema_for!(Schema).schema),
            ..Default::default()
        },
    )]
    .into_iter()
    .collect();

    response
}

impl CustomResponse {
    pub fn get_json_message(&self) -> String {
        let object = json!({
            "message": self.message.clone(),
        });

        schemars::_serde_json::to_string(&object).unwrap()
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for CustomResponse {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let message = self.get_json_message();
        let response = rocket::Response::build()
            .status(self.status)
            .header(rocket::http::ContentType::JSON)
            .streamed_body(std::io::Cursor::new(message))
            .finalize();

        Ok(response)
    }
}

impl OpenApiResponderInner for CustomResponse {
    fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Map::new();

        let status_codes = vec![
            Status::BadRequest,
            Status::Unauthorized,
            Status::Forbidden,
            Status::NotFound,
            Status::InternalServerError,
        ];

        for status in status_codes {
            responses.insert(
                status.code.to_string(),
                RefOr::Object(gen_response(status)),
            );
        }

        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}
