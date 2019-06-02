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

    use ::mystore_lib::models::product::{ Product, NewProduct };

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
                            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
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
                        web::resource("/products/{id}")
                            .route(web::get().to(::mystore_lib::handlers::products::show))
                            .route(web::delete().to(::mystore_lib::handlers::products::destroy))
                            .route(web::patch().to(::mystore_lib::handlers::products::update))
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

        let hat = NewProduct {
            name: Some("Hat".to_string()),
            stock: Some(15.0),
            price: Some(2045)
        };

        let pants = NewProduct {
            name: Some("Pants".to_string()),
            stock: Some(25.0),
            price: Some(3025)
        };
        let shoe_db = create_a_product(srv.borrow_mut(), csrf_token.clone(), request_cookie.clone(), &shoe);
        let hat_db = create_a_product(srv.borrow_mut(), csrf_token.clone(), request_cookie.clone(), &hat);
        create_a_product(srv.borrow_mut(), csrf_token.clone(), request_cookie.clone(), &pants);
        show_a_product(srv.borrow_mut(), csrf_token.clone(), request_cookie.clone(), &shoe_db.id, &shoe_db);
        let updated_hat = NewProduct {
            name: Some("Hat".to_string()),
            stock: Some(30.0),
            price: Some(3025)
        };
        update_a_product(srv.borrow_mut(), csrf_token.clone(), request_cookie.clone(), &hat_db.id, &updated_hat);
        products_index(srv.borrow_mut(), csrf_token, request_cookie, vec![shoe, updated_hat, pants]);
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
                            product: &NewProduct) -> Product {

        let request = srv
                          .post("/products")
                          .header(header::CONTENT_TYPE, "application/json")
                          .header("x-csrf-token", csrf_token.to_str().unwrap())
                          .cookie(request_cookie)
                          .timeout(std_duration::from_secs(600));

        let mut response =
            srv
                .block_on(request.send_body(json!(product).to_string()))
                .unwrap();

        assert!(response.status().is_success());

        let bytes = srv.block_on(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        serde_json::from_str(body).unwrap()
    }

    fn show_a_product(mut srv: RefMut<TestServerRuntime>,
                          csrf_token: HeaderValue,
                          request_cookie: Cookie,
                          id: &i32,
                          expected_product: &Product) {

        let request = srv
                        .get(format!("/products/{}", id))
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
        let response_product: Product = serde_json::from_str(body).unwrap();
        assert_eq!(&response_product, expected_product);
    }

    fn update_a_product(mut srv: RefMut<TestServerRuntime>,
                          csrf_token: HeaderValue,
                          request_cookie: Cookie,
                          id: &i32,
                          changes_to_product: &NewProduct) {

        let request = srv
                        .request(http::Method::PATCH, srv.url(&format!("/products/{}", id)))
                        .header(header::CONTENT_TYPE, "application/json")
                        .header("x-csrf-token", csrf_token.to_str().unwrap())
                        .cookie(request_cookie)
                        .timeout(std_duration::from_secs(600));

        let response =
            srv
                .block_on(request.send_body(json!(changes_to_product).to_string()))
                .unwrap();
        assert!(response.status().is_success());
    }

    fn products_index(mut srv: RefMut<TestServerRuntime>,
                          csrf_token: HeaderValue,
                          request_cookie: Cookie,
                      mut data_to_compare: Vec<NewProduct>) {

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
        let mut response_products: Vec<Product> = serde_json::from_str(body).unwrap();
        assert_eq!(data_to_compare.sort_by_key(|product| product.name.clone()), 
                   response_products.sort_by_key(|product| product.name.clone()));
    }

}