#[macro_use]
pub mod function_handler;
pub mod products;
pub mod prices;
pub mod register;
pub mod authentication;

use actix_web::{ Result, web };
use actix_web::HttpResponse;
use crate::db_connection::{ PgPool, PgPooledConnection };

pub fn pg_pool_handler(pool: web::Data<PgPool>) -> Result<PgPooledConnection> {
    pool
    .get()
    .map_err(|e| {
        actix_web::error::ErrorInternalServerError(e)
    })
}

use actix_web::{ FromRequest, HttpRequest, dev };
use actix_web::middleware::identity::Identity;
use crate::utils::jwt::{ decode_token, SlimUser };
pub type LoggedUser = SlimUser;

use hex;
use csrf_token::CsrfTokenGenerator;

impl FromRequest for LoggedUser {
    type Error = HttpResponse;
    type Config = ();
    type Future = Result<Self, HttpResponse>;

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        let generator = 
            req.app_data::<CsrfTokenGenerator>()
            .ok_or(HttpResponse::InternalServerError())?;
        
        let csrf_token =
            req
                .headers()
                .get("x-csrf-token")
                .ok_or(HttpResponse::Unauthorized())?;

        let decoded_token =
            hex::decode(&csrf_token)
                .map_err(|error| HttpResponse::InternalServerError().json(error.to_string()))?;

        generator
            .verify(&decoded_token)
            .map_err(|_| HttpResponse::Unauthorized())?;

        if let Some(identity) = Identity::from_request(req, payload)?.identity() {
            let user: SlimUser = decode_token(&identity)?;
            return Ok(user as LoggedUser);
        }  
        Err(HttpResponse::Unauthorized().into())
    }
}