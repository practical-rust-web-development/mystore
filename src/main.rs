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
use db_connection::establish_connection;

fn main() {
    let sys = actix::System::new("mystore");

    HttpServer::new(
    || App::new()
        .data(establish_connection())
        .service(
            web::resource("/products")
                .route(web::get().to_async(handlers::products::index))
                .route(web::post().to_async(handlers::products::create))
        )
        .service(
            web::resource("/products/{id}")
                .route(web::get().to_async(handlers::products::show))
                .route(web::delete().to_async(handlers::products::destroy))
                .route(web::patch().to_async(handlers::products::update))
        )
    )
    .bind("127.0.0.1:8088").unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8088");
    let _ = sys.run();
}