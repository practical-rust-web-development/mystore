use actix_web::{HttpRequest, HttpResponse };

use crate::models::product::{ NewProduct, ProductList };

pub fn index(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(ProductList::list())
}

use actix_web::web;

pub fn create(new_product: web::Json<NewProduct>) -> Result<HttpResponse, HttpResponse> {
    new_product
        .create()
        .map(|product| HttpResponse::Ok().json(product))
        .map_err(|e| {
            HttpResponse::InternalServerError().json(e.to_string())
        })
}