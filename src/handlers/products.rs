use actix_web::{HttpRequest, HttpResponse, Error, Responder};

use crate::models::product::ProductList;

impl Responder for ProductList {
    type Item = HttpResponse;
    type Error = Error;

    fn respond_to<S>(self, _req: &HttpRequest<S>) -> Result<HttpResponse, Error> {
        let body = serde_json::to_string(&self.0)?;

        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body))
    }
}

pub fn index(req: &HttpRequest) -> impl Responder {
    ProductList::list()
}