use actix_web::{ web, HttpResponse, Result };

use crate::models::product::ProductList;
use crate::handlers::LoggedUser;
use crate::db_connection::PgPool;
use crate::handlers::pg_pool_handler;

pub fn index(_user: LoggedUser, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    Ok(HttpResponse::Ok().json(ProductList::list(&pg_pool)))
}

use crate::models::product::NewProduct;
use crate::models::MyStoreResponder;
use actix_web::Responder;

pub fn create(_user: LoggedUser, new_product: web::Json<NewProduct>, pool: web::Data<PgPool>) ->
 impl Responder {
    let pg_pool = pg_pool_handler(pool)?;
    new_product
        .create(&pg_pool)
        .map(|product| MyStoreResponder(product))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}

use crate::models::product::Product;

pub fn show(_user: LoggedUser, id: web::Path<i32>, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    Product::find(&id, &pg_pool)
        .map(|product| HttpResponse::Ok().json(product))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}

pub fn destroy(_user: LoggedUser, id: web::Path<i32>, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    Product::destroy(&id, &pg_pool)
        .map(|_| HttpResponse::Ok().json(()))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}

pub fn update(_user: LoggedUser, id: web::Path<i32>, new_product: web::Json<NewProduct>, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    Product::update(&id, &new_product, &pg_pool)
        .map(|_| HttpResponse::Ok().json(()))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}