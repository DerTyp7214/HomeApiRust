use std::env;

use crypto::sha2::Sha256;
use jwt::{Header, Registered, Token};
use okapi::openapi3::{Object, SecurityRequirement, SecurityScheme, SecuritySchemeData};
use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};
use rocket_okapi::{
    gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};
use schemars::_serde_json;
use serde::{Deserialize, Serialize};

use crate::db::models::User;

static TOKEN_VERSION: &str = "1.0.0";

static mut STATIC_SECRET_KEY: Option<String> = None;

fn get_secret_key() -> String {
    unsafe {
        if STATIC_SECRET_KEY.is_none() {
            STATIC_SECRET_KEY = Some(env::var("SECRET_KEY").unwrap_or("".to_string()));
        }

        STATIC_SECRET_KEY.clone().unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JWTToken {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub token_version: String,
}

pub fn hash_password(password: &str) -> String {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap()
}

pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    bcrypt::verify(password, hashed_password).unwrap()
}

pub fn read_token(key: &str) -> Result<JWTToken, String> {
    let token = Token::<Header, Registered>::parse(key).map_err(|_| "Invalid token")?;
    if token.verify(get_secret_key().as_bytes(), Sha256::new()) {
        let sub = token.claims.sub.unwrap();
        let token_data: JWTToken = _serde_json::from_str(&sub).unwrap();
        Ok(token_data)
    } else {
        Err("Invalid token".into())
    }
}

pub fn create_token(token_data: JWTToken) -> String {
    let claims = Registered {
        sub: Some(_serde_json::to_string(&token_data).unwrap()),
        ..Default::default()
    };
    let token = Token::new(Header::default(), claims);
    token
        .signed(get_secret_key().as_bytes(), Sha256::new())
        .unwrap()
}

impl User {
    pub fn generate_token(&self) -> String {
        let token_data = JWTToken {
            user_id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            token_version: TOKEN_VERSION.to_owned(),
        };
        create_token(token_data)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JWTToken {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<JWTToken, Self::Error> {
        let keys: Vec<_> = request.headers().get("Authorization").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        let key = keys[0];
        let key = key.replace("Bearer ", "");
        match read_token(&key) {
            Ok(key) => Outcome::Success(key),
            Err(_) => Outcome::Failure((Status::Unauthorized, ())),
        }
    }
}

impl<'r> OpenApiFromRequest<'r> for JWTToken {
    fn from_request_input(
        _: &mut OpenApiGenerator,
        _: String,
        _: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        let mut requirements = SecurityRequirement::new();
        requirements.insert("HttpAuth".to_owned(), Vec::new());

        let input = RequestHeaderInput::Security(
            "HttpAuth".to_owned(),
            SecurityScheme {
                description: Some("API Key".to_string()),
                data: SecuritySchemeData::Http {
                    scheme: "bearer".to_owned(),
                    bearer_format: Some("bearer".to_owned()),
                },
                extensions: Object::default(),
            },
            requirements,
        );
        Ok(input)
    }
}
