#[macro_use]
extern crate dotenv_codegen;
extern crate itertools;
extern crate juniper;
extern crate diesel_derive_enum;

use actix_web::{App, HttpServer, web};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::http::header;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use csrf_token::CsrfTokenGenerator;
use chrono::Duration;
use ::mystore_lib::db_connection::establish_connection;

use ::mystore_lib::graphql::schema::create_schema;
use ::mystore_lib::graphql::graphql;

fn main() {
    std::env::set_var("RUST_LOG", "actix_web=debug");
    env_logger::init();
    let sys = actix::System::new("mystore");

    let csrf_token_header = header::HeaderName::from_lowercase(b"x-csrf-token").unwrap();

    let schema = std::sync::Arc::new(create_schema());

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
            Cors::new()
                .send_wildcard()
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
        .data(schema.clone())
        .service(
            web::resource("/prices")
                .route(web::get().to_async(::mystore_lib::handlers::prices::index))
                .route(web::post().to_async(::mystore_lib::handlers::prices::create))
        )
        .service(
            web::resource("/register")
                .route(web::post().to_async(::mystore_lib::handlers::register::register))
        )
        .service(
            web::resource("/auth")
                .route(web::post().to_async(::mystore_lib::handlers::authentication::login))
                .route(web::delete().to_async(::mystore_lib::handlers::authentication::logout))
        )
        .service(
            web::resource("/graphql").route(web::post().to_async(graphql))
        )
    )
    .bind("127.0.0.1:8088").unwrap()
    .start();

    println!("Started http server: 127.0.0.1:8088");
    let _ = sys.run();
}