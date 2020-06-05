use actix_identity::Identity;
use actix_web::HttpResponse;
use actix_web::{delete, post, web};
use csrf_token::CsrfTokenGenerator;
use hex;

use crate::db_connection::PgPool;
use crate::errors::MyStoreError;
use crate::handlers::pg_pool_handler;
use crate::models::user::AuthUser;
use crate::utils::jwt::create_token;

#[post("/login")]
pub async fn login(
    auth_user: web::Json<AuthUser>,
    id: Identity,
    pool: web::Data<PgPool>,
    generator: web::Data<CsrfTokenGenerator>,
) -> Result<HttpResponse, HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    let user = auth_user.login(&pg_pool).map_err(|e| match e {
        MyStoreError::DBError(diesel::result::Error::NotFound) => {
            HttpResponse::NotFound().json(e.to_string())
        }
        _ => HttpResponse::InternalServerError().json(e.to_string()),
    })?;

    let token = create_token(user.id, &user.email, &user.company)?;
    id.remember(token);
    let response = HttpResponse::Ok()
        .header("X-CSRF-TOKEN", hex::encode(generator.generate()))
        .json(user);
    Ok(response)
}

#[delete("/logout")]
pub async fn logout(id: Identity) -> Result<HttpResponse, HttpResponse> {
    id.forget();
    Ok(HttpResponse::Ok().json("success"))
}
