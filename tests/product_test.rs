#[macro_use]
extern crate dotenv_codegen;

mod common;

mod test{
    use actix_http::HttpService;
    use actix_http_test::{ TestServer, TestServerRuntime };
    use actix_web::http::header;
    use actix_identity::{CookieIdentityPolicy, IdentityService};
    use actix_web::{http, App, web};
    use actix_cors::Cors;
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

    use ::mystore_lib::models::product::{ Product, NewProduct, ProductList };
    use ::mystore_lib::models::user::{ NewUser, User };
    use ::mystore_lib::models::price::{ Price, PriceProduct, PriceProductToUpdate, NewPriceProduct, NewPrice };
    use ::mystore_lib::handlers::products::ProductWithPrices;

    #[test]
    fn test() {

        create_user();

        let csrf_token_header =
            header::HeaderName::from_lowercase(b"x-csrf-token").unwrap();

        let srv = RefCell::new(TestServer::new(move || 
            HttpService::new(
                App::new()
                    .wrap(
                        IdentityService::new(
                            CookieIdentityPolicy::new(dotenv!("SECRET_KEY").as_bytes())
                                .domain("localhost")
                                .name("mystorejwt")
                                .path("/")
                                .max_age(Duration::days(1).num_seconds())
                                .secure(false)
                        )
                    )
                    .wrap(
                        Cors::new()
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
                            dotenv!("CSRF_TOKEN_KEY").as_bytes().to_vec(),
                            Duration::hours(1)
                        )
                    )
                    .data(establish_connection())
                    .service(
                        web::resource("/products")
                            .route(web::get()
                                .to_async(::mystore_lib::handlers::products::index))
                            .route(web::post()
                                .to_async(::mystore_lib::handlers::products::create))
                    )
                    .service(
                        web::resource("/products/{id}")
                            .route(web::get()
                                .to_async(::mystore_lib::handlers::products::show))
                            .route(web::delete()
                                .to_async(::mystore_lib::handlers::products::destroy))
                            .route(web::patch()
                                .to_async(::mystore_lib::handlers::products::update))
                    )
                    .service(
                        web::resource("/prices")
                            .route(web::get()
                                .to_async(::mystore_lib::handlers::prices::index))
                            .route(web::post()
                                .to_async(::mystore_lib::handlers::prices::create))
                    )
                    .service(
                        web::resource("/auth")
                            .route(web::post()
                                .to_async(::mystore_lib::handlers::authentication::login))
                            .route(web::delete()
                                .to_async(::mystore_lib::handlers::authentication::logout))
                    )

            )
        ));

        let (csrf_token, request_cookie) = login(srv.borrow_mut());

        let shoe = NewProduct {
            name: Some("Shoe".to_string()),
            stock: Some(10.4),
            cost: Some(1892),
            description: Some("not just your regular shoes, this one will make you jump".to_string()),
            user_id: None
        };

        let hat = NewProduct {
            name: Some("Hat".to_string()),
            stock: Some(15.0),
            cost: Some(2045),
            description: Some("Just a regular hat".to_string()),
            user_id: None
        };

        let pants = NewProduct {
            name: Some("Pants".to_string()),
            stock: Some(25.0),
            cost: Some(3025),
            description: Some("beautiful black pants that will make you look thin".to_string()),
            user_id: None
        };

        let new_price_discount = NewPrice { name: Some("Discount".to_string()), user_id: None };
        let new_price_normal = NewPrice { name: Some("Normal".to_string()), user_id: None };

        let price_discount = create_a_price(srv.borrow_mut(),
                                            csrf_token.clone(),
                                            request_cookie.clone(),
                                            &new_price_discount);
        let price_normal = create_a_price(srv.borrow_mut(),
                                          csrf_token.clone(),
                                          request_cookie.clone(),
                                          &new_price_normal);

        let shoe_db = create_a_product(srv.borrow_mut(),
                                       csrf_token.clone(),
                                       request_cookie.clone(),
                                       &shoe,
                                       vec![&price_discount, &price_normal]);

        let hat_db = create_a_product(srv.borrow_mut(),
                                      csrf_token.clone(),
                                      request_cookie.clone(),
                                      &hat,
                                      vec![&price_discount, &price_normal]);

        let pants_db = create_a_product(srv.borrow_mut(),
                                        csrf_token.clone(), 
                                        request_cookie.clone(), 
                                        &pants,
                                        vec![&price_discount, &price_normal]);

        show_a_product(srv.borrow_mut(), 
                       csrf_token.clone(), 
                       request_cookie.clone(), 
                       &shoe_db.0.id, 
                       &shoe_db.0,
                       shoe_db.clone().1);
        let updated_hat = NewProduct {
            name: Some("Hat".to_string()),
            stock: Some(30.0),
            cost: Some(3025),
            description: Some("A hat with particular color, a dark black shining and beautiful".to_string()),
            user_id: None
        };
        update_a_product(srv.borrow_mut(), 
                         csrf_token.clone(), 
                         request_cookie.clone(), 
                         &hat_db.0.id, 
                         &updated_hat);
        destroy_a_product(srv.borrow_mut(), 
                          csrf_token.clone(), 
                          request_cookie.clone(), 
                          &pants_db.0.id);
        products_index(srv.borrow_mut(), 
                       csrf_token.clone(), 
                       request_cookie.clone(), 
                       vec![shoe.clone(), updated_hat.clone()]);
        search_products(srv.borrow_mut(), 
                        csrf_token, 
                        request_cookie, 
                        vec![updated_hat]);
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

    fn create_user() -> User {
        use diesel::RunQueryDsl;
        use ::mystore_lib::schema::users;
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
            .get_result::<User>(&pg_pool).unwrap()
    }

    fn create_a_product(mut srv: RefMut<TestServerRuntime>,
                            csrf_token: HeaderValue,
                            request_cookie: Cookie,
                            product: &NewProduct,
                            prices: Vec<&Price>) -> (Product, Vec<PriceProduct>) {

        let product_with_prices =
            ProductWithPrices {
                product: product.clone(),
                prices: vec![
                    PriceProductToUpdate {
                        price_product: NewPriceProduct { 
                            id: None,
                            price_id: prices.get(0).unwrap().id,  
                            product_id: None,
                            user_id: None,
                            amount: Some(4590)
                        },
                        to_delete: false
                    },
                    PriceProductToUpdate {
                        price_product: NewPriceProduct { 
                            id: None,
                            price_id: prices.get(1).unwrap().id,  
                            product_id: None,
                            user_id: None,
                            amount: Some(8709)
                        },
                        to_delete: false
                    }
                ]
            };

        let request = srv
                          .post("/products")
                          .header(header::CONTENT_TYPE, "application/json")
                          .header("x-csrf-token", csrf_token.to_str().unwrap())
                          .cookie(request_cookie)
                          .timeout(std_duration::from_secs(600));

        let mut response =
            srv
                .block_on(request.send_body(json!(product_with_prices).to_string()))
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
                          expected_product: &Product,
                          prices: Vec<PriceProduct>) {

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
        let response_product: (Product, Vec<PriceProduct>) = serde_json::from_str(body).unwrap();
        assert_eq!(response_product, (expected_product.clone(), prices));
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

        let product_with_prices =
            ProductWithPrices {
                product: changes_to_product.clone(),
                prices: vec![]
            };

        let response =
            srv
                .block_on(request.send_body(json!(product_with_prices).to_string()))
                .unwrap();

        assert!(response.status().is_success());
    }

    fn destroy_a_product(mut srv: RefMut<TestServerRuntime>,
                          csrf_token: HeaderValue,
                          request_cookie: Cookie,
                          id: &i32) {
        let request = srv
                        .request(http::Method::DELETE, srv.url(&format!("/products/{}", id)))
                        .header(header::CONTENT_TYPE, "application/json")
                        .header("x-csrf-token", csrf_token.to_str().unwrap())
                        .cookie(request_cookie)
                        .timeout(std_duration::from_secs(600));

        let response =
            srv
                .block_on(request.send())
                .unwrap();
        assert!(response.status().is_success());
    }

    fn products_index(mut srv: RefMut<TestServerRuntime>,
                          csrf_token: HeaderValue,
                          request_cookie: Cookie,
                      mut data_to_compare: Vec<NewProduct>) {

        let request = srv
                        .get("/products?search=&rank=100")
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
        let mut response_products: ProductList = serde_json::from_str(body).unwrap();
        data_to_compare.sort_by_key(|product| product.name.clone());
        response_products.0.sort_by_key(|product| product.0.name.clone());
        let products: Vec<Product> =
            response_products
            .0
            .iter()
            .map (|product| product.0.clone())
            .collect();
        assert_eq!(data_to_compare, products);
    }

    fn search_products(mut srv: RefMut<TestServerRuntime>,
                          csrf_token: HeaderValue,
                          request_cookie: Cookie,
                      mut data_to_compare: Vec<NewProduct>) {

        let request = srv
                        .get("/products?search=hats&rank=100")
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
        let mut response_products: ProductList = serde_json::from_str(body).unwrap();
        data_to_compare.sort_by_key(|product| product.name.clone());
        response_products.0.sort_by_key(|product| product.0.name.clone());
        let products: Vec<Product> =
            response_products
            .0
            .iter()
            .map (|product| product.0.clone())
            .collect();
        assert_eq!(data_to_compare, products);
    }

    fn create_a_price(mut srv: RefMut<TestServerRuntime>,
                          csrf_token: HeaderValue,
                          request_cookie: Cookie,
                          price: &NewPrice) -> Price {

        let request = srv
                          .post("/prices")
                          .header(header::CONTENT_TYPE, "application/json")
                          .header("x-csrf-token", csrf_token.to_str().unwrap())
                          .cookie(request_cookie)
                          .timeout(std_duration::from_secs(600));

        let mut response =
            srv
                .block_on(request.send_body(json!(price).to_string()))
                .unwrap();

        assert!(response.status().is_success());

        let bytes = srv.block_on(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        serde_json::from_str(body).unwrap()
    }
}