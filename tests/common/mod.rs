pub mod db_connection;

use ::mystore_lib::graphql::schema::create_schema;
use ::mystore_lib::graphql::{graphiql, graphql};
use actix_cors::Cors;
use actix_http::cookie::Cookie;
use actix_http::http::header::{HeaderValue, CONTENT_TYPE};
use actix_http::HttpService;
use actix_http_test::{test_server, TestServer};
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_service::map_config;
use actix_web::dev::AppConfig;
use actix_web::http::header;
use actix_web::App;
use chrono::Duration;
use csrf_token::CsrfTokenGenerator;
use serde_json::Value;
use std::cell::{RefCell, RefMut};
use std::str;
use std::time::Duration as std_duration;

use crate::common::db_connection::establish_connection;

pub fn server_test() -> RefCell<TestServer> {
    let schema = std::sync::Arc::new(create_schema());
    let csrf_token_header = header::HeaderName::from_lowercase(b"x-csrf-token").unwrap();

    RefCell::new(test_server(move || {
        HttpService::build()
            .h1(map_config(
                App::new()
                    .wrap(IdentityService::new(
                        CookieIdentityPolicy::new(dotenv!("SECRET_KEY").as_bytes())
                            .domain("localhost")
                            .name("mystorejwt")
                            .path("/")
                            .max_age(Duration::days(1).num_seconds())
                            .secure(false),
                    ))
                    .wrap(
                        Cors::new()
                            .allowed_origin("localhost")
                            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
                            .allowed_headers(vec![
                                header::AUTHORIZATION,
                                header::CONTENT_TYPE,
                                header::ACCEPT,
                                csrf_token_header.clone(),
                            ])
                            .expose_headers(vec![csrf_token_header.clone()])
                            .max_age(3600)
                            .finish(),
                    )
                    .data(CsrfTokenGenerator::new(
                        dotenv!("CSRF_TOKEN_KEY").as_bytes().to_vec(),
                        Duration::hours(1),
                    ))
                    .data(establish_connection())
                    .data(schema.clone())
                    .service(graphql)
                    .service(graphiql)
                    .service(::mystore_lib::handlers::authentication::login)
                    .service(::mystore_lib::handlers::authentication::logout),
                |_| AppConfig::default(),
            ))
            .tcp()
    }))
}

pub async fn send_request(
    srv: RefMut<'_, TestServer>,
    csrf_token: HeaderValue,
    request_cookie: Cookie<'_>,
    query: String,
) -> Value {
    let request = srv
        .post("/graphql")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-csrf-token", csrf_token.to_str().unwrap())
        .cookie(request_cookie)
        .timeout(std_duration::from_secs(600));

    let mut response = request.send_body(query).await.unwrap();

    assert!(response.status().is_success());

    assert_eq!(
        response.headers().get(CONTENT_TYPE).unwrap(),
        "application/json"
    );

    let bytes = response.body().await.unwrap();
    let body = str::from_utf8(&bytes).unwrap();
    serde_json::from_str(body).unwrap()
}
