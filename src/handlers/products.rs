use actix_web::{ web, HttpResponse, Result };

use crate::models::product::ProductList;
use crate::handlers::LoggedUser;
use crate::db_connection::PgPool;
use crate::handlers::pg_pool_handler;

#[derive(Deserialize)]
pub struct ProductSearch{ 
    pub search: String
}

#[derive(Deserialize)]
pub struct ProductPagination {
    pub rank: f64
}

pub fn index(user: LoggedUser,
             pool: web::Data<PgPool>,
             product_search: web::Query<ProductSearch>,
             pagination: web::Query<ProductPagination>) -> Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    let search = &product_search.search;
    Ok(HttpResponse::Ok().json(ProductList::list(user.id, search, pagination.rank, &pg_pool)))
}

use crate::models::product::NewProduct;

pub fn create(user: LoggedUser, new_product: web::Json<NewProduct>, pool: web::Data<PgPool>) ->
 Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;

    new_product
        .create(user.id, &pg_pool)
        .map(|product| HttpResponse::Ok().json(product))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}

use crate::models::product::Product;

pub fn show(user: LoggedUser, id: web::Path<i32>, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    Product::find(&id, user.id, &pg_pool)
        .map(|product| HttpResponse::Ok().json(product))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}

pub fn destroy(user: LoggedUser, id: web::Path<i32>, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    Product::destroy(&id, user.id, &pg_pool)
        .map(|_| HttpResponse::Ok().json(()))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}

pub fn update(user: LoggedUser, id: web::Path<i32>, new_product: web::Json<NewProduct>, pool: web::Data<PgPool>) -> Result<HttpResponse> {
    let pg_pool = pg_pool_handler(pool)?;
    Product::update(&id, user.id, &new_product, &pg_pool)
        .map(|_| HttpResponse::Ok().json(()))
        .map_err(|e| {
            actix_web::error::ErrorInternalServerError(e)
        })
}