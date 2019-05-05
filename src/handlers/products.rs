use actix_web::{HttpRequest, HttpResponse, Error, Responder};

use crate::models::product::Product;

impl Responder for Product {
    type Item = HttpResponse;
    type Error = Error;

    fn respond_to<S>(self, _req: &HttpRequest<S>) -> Result<HttpResponse, Error> {
        let body = serde_json::to_string(&Self::list())?;

        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body))
    }
}