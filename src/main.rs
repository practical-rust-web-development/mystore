pub mod schema;
pub mod db_connection;
pub mod models;
pub mod handlers;
pub mod errors;
pub mod utils;

#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate serde;
extern crate serde_json;
#[macro_use] 
extern crate serde_derive;

extern crate actix;
extern crate actix_web;
extern crate bcrypt;
extern crate jsonwebtoken as jwt;
extern crate csrf_token;

#[macro_use]
extern crate dotenv_codegen;

#[macro_use] extern crate log;
extern crate env_logger;

extern crate actix_http;

use actix_web::{App, HttpServer, web};
use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
use actix_web::http::header;
use actix_web::middleware::{cors, Logger};
use csrf_token::CsrfTokenGenerator;
use chrono::Duration;
use db_connection::establish_connection;

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();
    let sys = actix::System::new("mystore");

    let csrf_token_header = header::HeaderName::from_lowercase(b"x-csrf-token").unwrap();

    HttpServer::new(
    move || App::new()
        .wrap(Logger::default())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(dotenv!("SECRET_KEY").as_bytes())
                    .domain(dotenv!("MYSTOREDOMAIN"))
                    .name("mystorejwt")
                    .path("/")
                    .max_age(Duration::days(1).num_seconds())
                    .secure(dotenv!("COOKIE_SECURE").parse().unwrap())
            )
        )
        .wrap(
            cors::Cors::new()
                .allowed_origin(dotenv!("ALLOWED_ORIGIN"))
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec![header::AUTHORIZATION,
                                      header::CONTENT_TYPE,
                                      header::ACCEPT,
                                      csrf_token_header.clone()])
                .expose_headers(vec![csrf_token_header.clone()])
                .max_age(3600)
        )
        .data(
            CsrfTokenGenerator::new(
                dotenv!("CSRF_TOKEN_KEY").as_bytes().to_vec(),
                Duration::hours(1)
            )
        )
        .data(establish_connection())
        .service(
            web::resource("/products")
                .route(web::get().to(handlers::products::index))
                .route(web::post().to(handlers::products::create))
        )
        .service(
            web::resource("/products/{id}")
                .route(web::get().to(handlers::products::show))
                .route(web::delete().to(handlers::products::destroy))
                .route(web::patch().to(handlers::products::update))
        )
        .service(
            web::resource("/register")
                .route(web::post().to(handlers::register::register))
        )
        .service(
            web::resource("/auth")
                .route(web::post().to(handlers::authentication::login))
                .route(web::delete().to(handlers::authentication::logout))
        )
    )
    .bind("127.0.0.1:8088").unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8088");
    let _ = sys.run();
}