#[macro_use]
extern crate dotenv_codegen;
extern crate regex;

mod common;

mod test{
    use actix_http::HttpService;
    use actix_http_test::{ TestServer, TestServerRuntime };
    use actix_web::http::header;
    use actix_web::middleware::identity::{CookieIdentityPolicy, IdentityService};
    use actix_web::{http, App, web};
    use actix_web::middleware::cors;
    use chrono::Duration;
    use csrf_token::CsrfTokenGenerator;
    use actix_http::httpmessage::HttpMessage;
    use http::header::HeaderValue;
    use actix_http::cookie::Cookie;

    use serde_json::json;
    use std::str;
    use std::time::Duration as std_duration;
    use crate::common::db_connection::establish_connection;
    use std::cell::{ RefCell, RefMut };

    use ::mystore_lib::models::product::NewProduct;

    #[test]
    fn test() {

        create_user();

        let csrf_token_header = header::HeaderName::from_lowercase(b"x-csrf-token").unwrap();
        let srv = RefCell::new(TestServer::new(move || 
            HttpService::new(
                App::new()
                    .wrap(
                        IdentityService::new(
                            CookieIdentityPolicy::new("my very secure secret key for mystore".as_bytes())
                                .domain("localhost")
                                .name("mystorejwt")
                                .path("/")
                                .max_age(Duration::days(1).num_seconds())
                                .secure(false)
                        )
                    )
                    .wrap(
                        cors::Cors::new()
                            .allowed_origin("localhost")
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
                            "0123456789abcedf0123456789abcdef0123456789abcedf0123456789abcdef".as_bytes().to_vec(),
                            Duration::hours(1)
                        )
                    )
                    .data(establish_connection())
                    .service(
                        web::resource("/products")
                            .route(web::get().to(::mystore_lib::handlers::products::index))
                            .route(web::post().to(::mystore_lib::handlers::products::create))
                    )
                    .service(
                        web::resource("/auth")
                            .route(web::post().to(::mystore_lib::handlers::authentication::login))
                            .route(web::delete().to(::mystore_lib::handlers::authentication::logout))
                    )

            )
        ));

        let (csrf_token, request_cookie) = login(srv.borrow_mut());
        clear_products();

        let shoe = NewProduct {
            name: Some("Shoe".to_string()),
            stock: Some(10.4),
            price: Some(1892)
        };
        create_a_product(srv.borrow_mut(), csrf_token.clone(), request_cookie.clone(), shoe);
        products_index(srv.borrow_mut(), csrf_token, request_cookie)

    }

    fn login(mut srv: RefMut<TestServerRuntime>) -> (HeaderValue, Cookie) {
        let request = srv
                          .post("/auth")
                          .header(header::CONTENT_TYPE, "application/json")
                          .timeout(std_duration::from_secs(600));
        let response =
            srv
                .block_on(request.send_body(r#"{"email":"jhon@doe.com","password":"12345678"}"#))
                .unwrap();
        let csrf_token = response.headers().get("x-csrf-token").unwrap();
        let cookies = response.cookies().unwrap();
        let cookie = cookies[0].clone().into_owned().value().to_string();

        let request_cookie = Cookie::build("mystorejwt", cookie)
                                         .domain("localhost")
                                         .path("/")
                                         .max_age(Duration::days(1).num_seconds())
                                         .secure(false)
                                         .http_only(false)
                                         .finish();
        (csrf_token.clone(), request_cookie.clone())
    }

    fn create_user() {
        use diesel::RunQueryDsl;
        use ::mystore_lib::schema::users;
        use ::mystore_lib::models::user::{ NewUser, User };
        use chrono::Local;

        let connection = establish_connection();
        let pg_pool = connection.get().unwrap();

        diesel::delete(users::table).execute(&pg_pool).unwrap();

        diesel::insert_into(users::table)
            .values(NewUser {
                email: "jhon@doe.com".to_string(),
                company: "My own personal enterprise".to_string(),
                password: User::hash_password("12345678".to_string()).unwrap(),
                created_at: Local::now().naive_local()
            })
            .get_result::<User>(&pg_pool).unwrap();
    }

    fn clear_products() {
        use diesel::RunQueryDsl;
        use ::mystore_lib::schema::products;

        let connection = establish_connection();
        let pg_pool = connection.get().unwrap();
        diesel::delete(products::table).execute(&pg_pool).unwrap();
    }

    fn create_a_product(mut srv: RefMut<TestServerRuntime>,
                            csrf_token: HeaderValue,
                            request_cookie: Cookie,
                            product: NewProduct) {

        let request = srv
                          .post("/products")
                          .header(header::CONTENT_TYPE, "application/json")
                          .header("x-csrf-token", csrf_token.to_str().unwrap())
                          .cookie(request_cookie)
                          .timeout(std_duration::from_secs(600));

        let response =
            srv
                .block_on(request.send_body(json!(product).to_string()))
                .unwrap();

        assert!(response.status().is_success());
    }

    fn products_index(mut srv: RefMut<TestServerRuntime>, csrf_token: HeaderValue, request_cookie: Cookie) {
        use regex::Regex;

        let request = srv
                        .get("/products")
                        .header("x-csrf-token", csrf_token.to_str().unwrap())
                        .cookie(request_cookie);

        let mut response = srv.block_on(request.send()).unwrap();
        assert!(response.status().is_success());

        assert_eq!(
            response.headers().get(http::header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let bytes = srv.block_on(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        let re = Regex::new(r#"[{"id":\d*,"name":"Show","stock":10.4,"price":1892}]"#).unwrap();
        assert!(re.is_match(body));
    }

}