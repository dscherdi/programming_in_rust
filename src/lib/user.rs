use rocket::{
    outcome::IntoOutcome,
    request::{self, FromRequest, Request}
};
use std::str::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UserData {
	username: String,
	password: String,
    pub group: String,
    pub role: Role,
}

#[derive(rocket::FromForm)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Role {
    Standard,
    Elevated,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCookie {
    pub group: String,
    pub role: Role,
}

impl PartialEq<UserData> for LoginData {
    fn eq(&self, other: &UserData) -> bool {
        self.username == other.username &&
        self.password == other.password
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserCookie {
    type Error = std::convert::Infallible;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<UserCookie, Self::Error> {
        req.cookies()
            .get_private("user")
            .and_then(|cookie| match serde_json::from_str(cookie.value()) {
                Ok(u) => Some(u),
                Err(_) => None
            })
            .or_forward(())
    }
}

impl FromStr for UserCookie {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match serde_json::from_str::<Self>(s) {
            Ok(u) => Ok(u),
            Err(_) => Err(())
        }
    }
}