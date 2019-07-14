use actix_web::{ web, HttpResponse, Result };

use crate::models::price::PriceList;
use crate::handlers::LoggedUser;
use crate::db_connection::PgPool;
use crate::handlers::pg_pool_handler;

pub fn index(user: LoggedUser, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;

    PriceList::list(user.id, &pg_pool)
        .map(|prices| HttpResponse::Ok().json(prices))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}

use crate::models::price::NewPrice;

pub fn create(user: LoggedUser,
              new_price: web::Json<NewPrice>,
              pool: web::Data<PgPool>) ->
 Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;

    new_price
        .create(user.id, &pg_pool)
        .map(|price| HttpResponse::Ok().json(price))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}

use crate::models::price::Price;

pub fn show(user: LoggedUser, id: web::Path<i32>, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    Price::find(&id, user.id, &pg_pool)
        .map(|price| HttpResponse::Ok().json(price))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}

pub fn destroy(user: LoggedUser, id: web::Path<i32>, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    Price::destroy(&id, user.id, &pg_pool)
        .map(|_| HttpResponse::Ok().json(()))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}

pub fn update(user: LoggedUser,
              id: web::Path<i32>,
              new_price: web::Json<NewPrice>,
              pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    Price::update(user.id, *id, new_price.clone(), &pg_pool)
        .map(|_| HttpResponse::Ok().json(()))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}
