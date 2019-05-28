use actix_web::web;
use crate::db_connection::PgPool;
use actix_web::HttpResponse;
use crate::handlers::pg_pool_handler;

use crate::models::user::{ User, RegisterUser };

pub fn register(new_user: web::Json<RegisterUser>, pool: web::Data<PgPool>) ->
 Result<HttpResponse, HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    let register_user = new_user
        .into_inner()
        .validates()
        .map_err(|e| {
           HttpResponse::InternalServerError().json(e.to_string())
        })?;
    User::create(register_user, &pg_pool)
        .map(|user| HttpResponse::Ok().json(user))
        .map_err(|e| {
           HttpResponse::InternalServerError().json(e.to_string())
        })
}
