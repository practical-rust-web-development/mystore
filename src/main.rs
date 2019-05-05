pub mod schema;
pub mod db_connection;
pub mod models;
pub mod handlers;

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate serde;
extern crate serde_json;
#[macro_use] 
extern crate serde_derive;

extern crate actix_web;
use actix_web::{server, App, HttpRequest};

fn index(_req: &HttpRequest) -> &'static str {
    "Hello world!"
}

fn main() {
    server::new(|| App::new().resource("/", |r| r.f(index)))
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}