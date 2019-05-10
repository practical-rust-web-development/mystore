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
extern crate futures;
use actix_web::{App, HttpServer, web};

fn main() {
    let sys = actix::System::new("mystore");

    HttpServer::new(
    || App::new()
        .service(
            web::resource("/products")
                .route(web::get().to_async(handlers::products::index))
                .route(web::post().to_async(handlers::products::create))
        ))
    .bind("127.0.0.1:8088").unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8088");
    let _ = sys.run();
}