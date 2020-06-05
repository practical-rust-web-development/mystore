#[macro_use]
pub mod register;
pub mod authentication;

use actix_identity::Identity;
use actix_web::error::{ErrorBadRequest, ErrorUnauthorized};
use actix_web::{dev, FromRequest, HttpRequest};
use actix_web::{web, Error, Result};
use chrono::Duration;
use csrf_token::CsrfTokenGenerator;
use futures_util::future::{err, ok, Ready};
use hex;

use crate::db_connection::{PgPool, PgPooledConnection};
use crate::utils::jwt::{decode_token, SlimUser};

pub type LoggedUser = SlimUser;

pub fn pg_pool_handler(pool: web::Data<PgPool>) -> Result<PgPooledConnection> {
    pool.get()
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))
}

impl FromRequest for LoggedUser {
    type Error = Error;
    type Config = ();
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        match get_token(req, payload) {
            Ok(user) => ok(user),
            Err(error) => err(error),
        }
    }
}

fn get_token(req: &HttpRequest, payload: &mut dev::Payload) -> Result<LoggedUser, Error> {
    let generator = CsrfTokenGenerator::new(
        dotenv!("CSRF_TOKEN_KEY").as_bytes().to_vec(),
        Duration::hours(1),
    );

    let csrf_token = req
        .headers()
        .get("x-csrf-token")
        .ok_or(ErrorBadRequest("No token provided"))?;

    let decoded_token = hex::decode(&csrf_token)
        .map_err(|_| ErrorBadRequest("An Error ocurred decoding the token"))?;

    generator
        .verify(&decoded_token)
        .map_err(|_| ErrorUnauthorized("can't verify token"))?;

    if let Some(identity) = Identity::from_request(req, payload)
        .into_inner()?
        .identity()
    {
        let user: SlimUser = decode_token(&identity)?;
        Ok(user as LoggedUser)
    } else {
        Err(ErrorUnauthorized("can't obtain token"))
    }
}
