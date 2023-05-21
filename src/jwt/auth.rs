use dotenv::dotenv;
use rocket_jwt::jwt;

static SECRET_KEY: &str = "";
static JWT_ISSUER: &str = "";

/*#[jwt(SECRET_KEY)]
pub struct User {} TODO: fix this
*/ 