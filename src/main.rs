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

extern crate actix;
extern crate actix_web;
use actix_web::{server, App, http};

fn main() {
    let sys = actix::System::new("mystore");

    server::new(
    || App::new()
        .resource("/products", |r| r.method(http::Method::GET).f(handlers::products::index)))
    .bind("127.0.0.1:8088").unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8088");
    let _ = sys.run();
}