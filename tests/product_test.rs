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

    use serde_json::{json, Value};
    use std::str;
    use std::time::Duration as std_duration;
    use crate::common::db_connection::establish_connection;
    use std::cell::{RefCell, RefMut};

    use ::mystore_lib::models::product::{Product, NewProduct, FullProduct};
    use ::mystore_lib::models::user::{NewUser, User};
    use ::mystore_lib::models::price::{Price, 
        PriceProduct, 
        PriceProductToUpdate, 
        NewPriceProduct, 
        NewPrice, 
        NewPriceProductsToUpdate};
    use ::mystore_lib::graphql::schema::create_schema;
    use ::mystore_lib::graphql::graphql;

    #[test]
    fn test() {

        create_user();

        let csrf_token_header =
            header::HeaderName::from_lowercase(b"x-csrf-token").unwrap();

        let schema = std::sync::Arc::new(create_schema());

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
                    .data(schema.clone())
                    .service(
                        web::resource("/graphql").route(web::post().to_async(graphql))
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
            id: None,
            name: Some("Shoe".to_string()),
            stock: Some(10.4),
            cost: Some(1892),
            description: Some("not just your regular shoes, this one will make you jump".to_string()),
            user_id: None
        };

        let hat = NewProduct {
            id: None,
            name: Some("Hat".to_string()),
            stock: Some(15.0),
            cost: Some(2045),
            description: Some("Just a regular hat".to_string()),
            user_id: None
        };

        let pants = NewProduct {
            id: None,
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

        let all_prices = NewPriceProductsToUpdate {
            data: vec![
                PriceProductToUpdate {
                    to_delete: false,
                    price_product: NewPriceProduct {
                        id: None,
                        product_id: None,
                        user_id: None,
                        price_id: price_discount.clone().id,
                        amount: Some(10)
                    }
                },
                PriceProductToUpdate {
                    to_delete: false,
                    price_product: NewPriceProduct {
                        id: None,
                        product_id: None,
                        user_id: None,
                        price_id: price_normal.clone().id,
                        amount: Some(15)
                    }
                }
            ]
        };

        let response_shoe_db = create_a_product(srv.borrow_mut(),
                                       csrf_token.clone(),
                                       request_cookie.clone(),
                                       &shoe,
                                       all_prices.clone());

        let shoe_db = response_shoe_db.get("data").unwrap().get("createProduct").unwrap();
        let shoe_id: i32 = serde_json::from_value(shoe_db.get("product").unwrap().get("id").unwrap().clone()).unwrap();

        let response_hat_db = create_a_product(srv.borrow_mut(),
                                      csrf_token.clone(),
                                      request_cookie.clone(),
                                      &hat,
                                      all_prices.clone());

        let hat_db = response_hat_db.get("data").unwrap().get("createProduct").unwrap();
        let hat_id: i32 = serde_json::from_value(hat_db.get("product").unwrap().get("id").unwrap().clone()).unwrap();

        let response_pants_db = create_a_product(srv.borrow_mut(),
                                        csrf_token.clone(), 
                                        request_cookie.clone(), 
                                        &pants,
                                        all_prices.clone());

        let pants_db = response_pants_db.get("data").unwrap().get("createProduct").unwrap();

        show_a_product(srv.borrow_mut(), 
                       csrf_token.clone(), 
                       request_cookie.clone(), 
                       shoe_id, 
                       &shoe_db);

        let updated_hat = NewProduct {
            id: None,
            name: Some("Hat".to_string()),
            stock: Some(30.0),
            cost: Some(3025),
            description: Some("A hat with particular color, a dark black shining and beautiful".to_string()),
            user_id: None
        };

        update_a_product(srv.borrow_mut(), 
                         csrf_token.clone(), 
                         request_cookie.clone(), 
                         &hat_id, 
                         &updated_hat,
                         all_prices.clone());

        destroy_a_product(srv.borrow_mut(), 
                          csrf_token.clone(), 
                          request_cookie.clone(), 
                          &pants_db.0.id);

        //products_index(srv.borrow_mut(), 
        //               csrf_token.clone(), 
        //               request_cookie.clone(), 
        //               vec![shoe.clone(), updated_hat.clone()]);
        //search_products(srv.borrow_mut(), 
        //                csrf_token, 
        //                request_cookie, 
        //                vec![updated_hat]);
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
                            prices: NewPriceProductsToUpdate) -> Value {
        
        let request = srv
                          .post("/graphql")
                          .header(header::CONTENT_TYPE, "application/json")
                          .header("x-csrf-token", csrf_token.to_str().unwrap())
                          .cookie(request_cookie)
                          .timeout(std_duration::from_secs(600));

        let prices_to_s: Vec<String> = prices.data.iter().map(|price| {
            format!(
                r#"
                {{
                    "toDelete": {},
                    "priceProduct": {{
                        "priceId": {},
                        "amount": {}
                    }}
                }}"#,
                false,
                price.price_product.price_id,
                price.price_product.amount.unwrap()
            )
        }).collect();

        let query =
            format!(
            r#"
            {{
                "query": "
                    mutation CreateProduct($paramNewProduct: NewProduct!, $paramNewPriceProducts: NewPriceProductsToUpdate!) {{
                            createProduct(paramNewProduct: $paramNewProduct, paramNewPriceProducts: $paramNewPriceProducts) {{
                                product {{
                                    id
                                    name
                                    stock
                                    cost
                                    description
                                    userId
                                }}
                                priceProducts {{
                                    priceProduct {{
                                        id
                                        priceId
                                        userId
                                        amount
                                    }}
                                    price {{
                                        id
                                        name
                                        userId
                                    }}
                                }}
                            }}
                    }}
                ",
                "variables": {{
                    "paramNewProduct": {{
                        "name": "{}",
                        "stock": {},
                        "cost": {},
                        "description": "{}"
                    }},
                    "paramNewPriceProducts": {{ "data": [{}] }}
                }}
            }}"#,
            product.clone().name.unwrap(),
            product.clone().stock.unwrap(),
            product.clone().cost.unwrap(),
            product.clone().description.unwrap(),
            prices_to_s.join(","))
            .replace("\n", "");

        let mut response =
            srv
                .block_on(request.send_body(query))
                .unwrap();

        assert!(response.status().is_success());

        let bytes = srv.block_on(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        serde_json::from_str(body).unwrap()
    }

    fn show_a_product(mut srv: RefMut<TestServerRuntime>,
                          csrf_token: HeaderValue,
                          request_cookie: Cookie,
                          id: i32,
                          expected_product: &Value) {

        let query = format!(r#"
            {{
                "query": "
                    query ShowAProduct($productId: Int!) {{
                        product(productId: $productId) {{
                            product {{
                                id
                                name
                                stock
                                cost
                                description
                                userId
                            }}
                            priceProducts {{
                                priceProduct {{
                                    id
                                    priceId
                                    userId
                                    amount
                                }}
                                price {{
                                    id
                                    name
                                    userId
                                }}
                            }}
                        }}
                    }}
                ",
                "variables": {{
                    "productId": {}
                }}
            }}
        "#, id).replace("\n", "");

        let request = srv
                          .post("/graphql")
                          .header(header::CONTENT_TYPE, "application/json")
                          .header("x-csrf-token", csrf_token.to_str().unwrap())
                          .cookie(request_cookie)
                          .timeout(std_duration::from_secs(600));

        let mut response =
            srv
                .block_on(request.send_body(query))
                .unwrap();
        assert!(response.status().is_success());

        assert_eq!(
            response.headers().get(http::header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let bytes = srv.block_on(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        let response_product: Value = serde_json::from_str(body).unwrap();
        let product = response_product.get("data").unwrap().get("product").unwrap();
        assert_eq!(product, expected_product);
    }

    fn update_a_product(mut srv: RefMut<TestServerRuntime>,
                          csrf_token: HeaderValue,
                          request_cookie: Cookie,
                          id: &i32,
                          changes_to_product: &NewProduct,
                          prices: NewPriceProductsToUpdate) -> Value {

        let prices_to_s: Vec<String> = prices.data.iter().map(|price| {
            format!(
                r#"
                {{
                    "toDelete": {},
                    "priceProduct": {{
                        "priceId": {},
                        "amount": {}
                    }}
                }}"#,
                false,
                price.price_product.price_id,
                price.price_product.amount.unwrap()
            )
        }).collect();

        let query = 
            format!(
            r#"
            {{
                "query": "
                    mutation UpdateProduct($paramNewProduct: NewProduct!, $paramNewPriceProducts: NewPriceProductsToUpdate!) {{
                            updateProduct(paramNewProduct: $paramNewProduct, paramNewPriceProducts: $paramNewPriceProducts) {{
                                product {{
                                    id
                                    name
                                    stock
                                    cost
                                    description
                                    userId
                                }}
                                priceProducts {{
                                    priceProduct {{
                                        id
                                        priceId
                                        userId
                                        amount
                                    }}
                                    price {{
                                        id
                                        name
                                        userId
                                    }}
                                }}
                            }}
                    }}
                ",
                "variables": {{
                    "paramNewProduct": {{
                        "name": "{}",
                        "stock": {},
                        "cost": {},
                        "description": "{}"
                    }},
                    "paramNewPriceProducts": {{ "data": [{}] }}
                }}
            }}"#,
            changes_to_product.clone().name.unwrap(),
            changes_to_product.clone().stock.unwrap(),
            changes_to_product.clone().cost.unwrap(),
            changes_to_product.clone().description.unwrap(),
            prices_to_s.join(","))
            .replace("\n", "");

        let request = srv
                          .post("/graphql")
                          .header(header::CONTENT_TYPE, "application/json")
                          .header("x-csrf-token", csrf_token.to_str().unwrap())
                          .cookie(request_cookie)
                          .timeout(std_duration::from_secs(600));


        let mut response =
            srv
                .block_on(request.send_body(query))
                .unwrap();

        assert!(response.status().is_success());

        let bytes = srv.block_on(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        serde_json::from_str(body).unwrap()
    }

    //fn destroy_a_product(mut srv: RefMut<TestServerRuntime>,
    //                      csrf_token: HeaderValue,
    //                      request_cookie: Cookie,
    //                      id: &i32) {
    //    let request = srv
    //                    .request(http::Method::DELETE, srv.url(&format!("/products/{}", id)))
    //                    .header(header::CONTENT_TYPE, "application/json")
    //                    .header("x-csrf-token", csrf_token.to_str().unwrap())
    //                    .cookie(request_cookie)
    //                    .timeout(std_duration::from_secs(600));

    //    let response =
    //        srv
    //            .block_on(request.send())
    //            .unwrap();
    //    assert!(response.status().is_success());
    //}

    //fn products_index(mut srv: RefMut<TestServerRuntime>,
    //                      csrf_token: HeaderValue,
    //                      request_cookie: Cookie,
    //                  mut data_to_compare: Vec<NewProduct>) {

    //    let request = srv
    //                    .get("/products?search=&rank=100")
    //                    .header("x-csrf-token", csrf_token.to_str().unwrap())
    //                    .cookie(request_cookie);

    //    let mut response = srv.block_on(request.send()).unwrap();
    //    assert!(response.status().is_success());

    //    assert_eq!(
    //        response.headers().get(http::header::CONTENT_TYPE).unwrap(),
    //        "application/json"
    //    );

    //    let bytes = srv.block_on(response.body()).unwrap();
    //    let body = str::from_utf8(&bytes).unwrap();
    //    let mut response_products: ProductList = serde_json::from_str(body).unwrap();
    //    data_to_compare.sort_by_key(|product| product.name.clone());
    //    response_products.0.sort_by_key(|product| product.0.name.clone());
    //    let products: Vec<Product> =
    //        response_products
    //        .0
    //        .iter()
    //        .map (|product| product.0.clone())
    //        .collect();
    //    assert_eq!(data_to_compare, products);
    //}

    //fn search_products(mut srv: RefMut<TestServerRuntime>,
    //                      csrf_token: HeaderValue,
    //                      request_cookie: Cookie,
    //                  mut data_to_compare: Vec<NewProduct>) {

    //    let request = srv
    //                    .get("/products?search=hats&rank=100")
    //                    .header("x-csrf-token", csrf_token.to_str().unwrap())
    //                    .cookie(request_cookie);

    //    let mut response = srv.block_on(request.send()).unwrap();
    //    assert!(response.status().is_success());

    //    assert_eq!(
    //        response.headers().get(http::header::CONTENT_TYPE).unwrap(),
    //        "application/json"
    //    );

    //    let bytes = srv.block_on(response.body()).unwrap();
    //    let body = str::from_utf8(&bytes).unwrap();
    //    let mut response_products: ProductList = serde_json::from_str(body).unwrap();
    //    data_to_compare.sort_by_key(|product| product.name.clone());
    //    response_products.0.sort_by_key(|product| product.0.name.clone());
    //    let products: Vec<Product> =
    //        response_products
    //        .0
    //        .iter()
    //        .map (|product| product.0.clone())
    //        .collect();
    //    assert_eq!(data_to_compare, products);
    //}

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