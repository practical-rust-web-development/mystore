#[macro_use]
extern crate dotenv_codegen;

mod common;

mod test{
    use actix_http::HttpService;
    use actix_http_test::{ TestServer, test_server };
    use actix_web::http::header;
    use actix_identity::{CookieIdentityPolicy, IdentityService};
    use actix_web::{http, App};
    use actix_cors::Cors;
    use chrono::Duration;
    use csrf_token::CsrfTokenGenerator;
    use actix_http::httpmessage::HttpMessage;
    use http::header::HeaderValue;
    use actix_http::cookie::Cookie;

    use actix_service::map_config;
    use actix_web::dev::AppConfig;
    use serde_json::{json, Value};
    use std::str;
    use std::time::Duration as std_duration;
    use crate::common::db_connection::establish_connection;
    use std::cell::{RefCell, RefMut};

    use ::mystore_lib::models::product::{FormProduct};
    use ::mystore_lib::models::user::{NewUser, User};
    use ::mystore_lib::models::price::{ 
        PriceProductToUpdate, 
        FormPriceProduct, 
        FormPrice, 
        FormPriceProductsToUpdate};
    use ::mystore_lib::graphql::schema::create_schema;
    use ::mystore_lib::graphql::{graphql, graphiql};

    #[actix_rt::test]
    async fn test() {

        create_user();

        let csrf_token_header =
            header::HeaderName::from_lowercase(b"x-csrf-token").unwrap();

        let schema = std::sync::Arc::new(create_schema());

        let srv = RefCell::new(test_server(move || {
            HttpService::build()
                .h1(map_config(
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
                                .finish()
                        )
                        .data(
                            CsrfTokenGenerator::new(
                                dotenv!("CSRF_TOKEN_KEY").as_bytes().to_vec(),
                                Duration::hours(1)
                            )
                        )
                        .data(establish_connection())
                        .data(schema.clone())
                        .service(graphql)
                        .service(graphiql)
                        .service(::mystore_lib::handlers::authentication::login)
                        .service(::mystore_lib::handlers::authentication::logout)
                    ,  |_| AppConfig::default(),
                ))
                .tcp()
            }
        ));

        let (csrf_token, request_cookie) = login(srv.borrow_mut()).await;

        let shoe = FormProduct {
            id: None,
            name: Some("Shoe".to_string()),
            stock: Some(10.4),
            cost: Some(1892),
            description: Some("not just your regular shoes, this one will make you jump".to_string()),
            user_id: None
        };

        let hat = FormProduct {
            id: None,
            name: Some("Hat".to_string()),
            stock: Some(15.0),
            cost: Some(2045),
            description: Some("Just a regular hat".to_string()),
            user_id: None
        };

        let pants = FormProduct {
            id: None,
            name: Some("Pants".to_string()),
            stock: Some(25.0),
            cost: Some(3025),
            description: Some("beautiful black pants that will make you look thin".to_string()),
            user_id: None
        };

        let new_price_discount = FormPrice { id: None, name: Some("Discount".to_string()), user_id: None };
        let new_price_normal = FormPrice { id: None, name: Some("Normal".to_string()), user_id: None };

        let price_discount = create_a_price(srv.borrow_mut(),
                                            csrf_token.clone(),
                                            request_cookie.clone(),
                                            &new_price_discount).await;
        let price_normal = create_a_price(srv.borrow_mut(),
                                          csrf_token.clone(),
                                          request_cookie.clone(),
                                          &new_price_normal).await;

        let price_discount_db = price_discount.get("data").unwrap().get("createPrice").unwrap();
        let price_discount_id: i32 = serde_json::from_value(price_discount_db.get("id").unwrap().clone()).unwrap();

        let price_normal_db = price_normal.get("data").unwrap().get("createPrice").unwrap();
        let price_normal_id: i32 = serde_json::from_value(price_normal_db.get("id").unwrap().clone()).unwrap();

        let all_prices = FormPriceProductsToUpdate {
            data: vec![
                PriceProductToUpdate {
                    to_delete: false,
                    price_product: FormPriceProduct {
                        id: None,
                        product_id: None,
                        user_id: None,
                        price_id: price_discount_id,
                        amount: Some(10)
                    }
                },
                PriceProductToUpdate {
                    to_delete: false,
                    price_product: FormPriceProduct {
                        id: None,
                        product_id: None,
                        user_id: None,
                        price_id: price_normal_id,
                        amount: Some(15)
                    }
                }
            ]
        };

        let response_shoe_db = create_a_product(srv.borrow_mut(),
                                                csrf_token.clone(),
                                                request_cookie.clone(),
                                                &shoe,
                                                all_prices.clone()).await;

        let shoe_db = response_shoe_db.get("data").unwrap().get("createProduct").unwrap();
        let shoe_id: i32 = serde_json::from_value(shoe_db.get("product").unwrap().get("id").unwrap().clone()).unwrap();

        let response_hat_db = create_a_product(srv.borrow_mut(),
                                               csrf_token.clone(),
                                               request_cookie.clone(),
                                               &hat,
                                               all_prices.clone()).await;

        let hat_db = response_hat_db.get("data").unwrap().get("createProduct").unwrap();
        let hat_id: i32 = serde_json::from_value(hat_db.get("product").unwrap().get("id").unwrap().clone()).unwrap();

        let response_pants_db = create_a_product(srv.borrow_mut(),
                                                 csrf_token.clone(), 
                                                 request_cookie.clone(), 
                                                 &pants,
                                                 all_prices.clone()).await;

        let pants_db = response_pants_db.get("data").unwrap().get("createProduct").unwrap();
        let pants_id: i32 = serde_json::from_value(pants_db.get("product").unwrap().get("id").unwrap().clone()).unwrap();

        show_a_product(srv.borrow_mut(), 
                       csrf_token.clone(), 
                       request_cookie.clone(), 
                       shoe_id, 
                       &shoe_db).await;

        let updated_hat = FormProduct {
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
                         &updated_hat,
                         all_prices.clone()).await;

        let response_product_destroyed = 
            destroy_a_product(srv.borrow_mut(), 
                              csrf_token.clone(), 
                              request_cookie.clone(), 
                              &pants_id).await;
        
        let destroyed: bool =
            serde_json::from_value(
                response_product_destroyed
                    .get("data")
                    .unwrap()
                    .get("destroyProduct")
                    .unwrap()
                    .clone()
            ).unwrap();
        assert!(destroyed);

        let data_for_searching = json!({
            "data": {
                "listProduct": {
                    "data": [{
                        "priceProducts": [
                            {
                                "price": {
                                    "name": "Discount"
                                },
                                "priceProduct": {
                                    "amount": 10
                                }
                            },
                            {
                                "price": {
                                    "name": "Normal"
                                },
                                "priceProduct": {
                                    "amount": 15
                                }
                            }
                        ],
                        "product": {
                            "cost": 2045,
                            "name": "Hat",
                            "description": "Just a regular hat",
                            "id": hat_id,
                            "stock": 15.0
                        }
                    }]
                }
            }
        });

        search_products(srv.borrow_mut(), 
                        csrf_token, 
                        request_cookie, 
                        data_for_searching).await;
    }

    async fn login(srv: RefMut<'_, TestServer>) -> (HeaderValue, Cookie<'_>) {
        let request = srv
                          .post("/login")
                          .header(header::CONTENT_TYPE, "application/json")
                          .timeout(std_duration::from_secs(600));
        let response =
            request
                .send_body(r#"{"email":"jhon@doe.com","password":"12345678"}"#)
                .await
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

    async fn create_a_product(srv: RefMut<'_, TestServer>,
                              csrf_token: HeaderValue,
                              request_cookie: Cookie<'_>,
                              product: &FormProduct,
                              prices: FormPriceProductsToUpdate) -> Value {
        
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
                    mutation CreateProduct($form: FormProduct!, $formPriceProducts: FormPriceProductsToUpdate!) {{
                            createProduct(form: $form, formPriceProducts: $formPriceProducts) {{
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
                    "form": {{
                        "name": "{}",
                        "stock": {},
                        "cost": {},
                        "description": "{}"
                    }},
                    "formPriceProducts": {{ "data": [{}] }}
                }}
            }}"#,
            product.clone().name.unwrap(),
            product.clone().stock.unwrap(),
            product.clone().cost.unwrap(),
            product.clone().description.unwrap(),
            prices_to_s.join(","))
            .replace("\n", "");

        let mut response =
            request
                .send_body(query)
                .await
                .unwrap();

        assert!(response.status().is_success());

        let bytes = response.body().await.unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        serde_json::from_str(body).unwrap()
    }

    async fn show_a_product(srv: RefMut<'_, TestServer>,
                            csrf_token: HeaderValue,
                            request_cookie: Cookie<'_>,
                            id: i32,
                            expected_product: &Value) {

        let query = format!(r#"
            {{
                "query": "
                    query ShowProduct($productId: Int!) {{
                        showProduct(productId: $productId) {{
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
            request
                .send_body(query)
                .await
                .unwrap();
        assert!(response.status().is_success());

        assert_eq!(
            response.headers().get(http::header::CONTENT_TYPE).unwrap(),
            "application/json"
        );

        let bytes = response.body().await.unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        let response_product: Value = serde_json::from_str(body).unwrap();
        let product = response_product.get("data").unwrap().get("showProduct").unwrap();
        assert_eq!(product, expected_product);
    }

    async fn update_a_product(srv: RefMut<'_, TestServer>,
                              csrf_token: HeaderValue,
                              request_cookie: Cookie<'_>,
                              changes_to_product: &FormProduct,
                              prices: FormPriceProductsToUpdate) -> Value {

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
                    mutation UpdateProduct($paramFormProduct: FormProduct!, $paramFormPriceProducts: FormPriceProductsToUpdate!) {{
                            updateProduct(paramFormProduct: $paramFormProduct, paramFormPriceProducts: $paramFormPriceProducts) {{
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
                    "paramFormProduct": {{
                        "name": "{}",
                        "stock": {},
                        "cost": {},
                        "description": "{}"
                    }},
                    "paramFormPriceProducts": {{ "data": [{}] }}
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
            request
                .send_body(query)
                .await
                .unwrap();

        assert!(response.status().is_success());

        let bytes = response.body().await.unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        serde_json::from_str(body).unwrap()
    }

    async fn destroy_a_product(srv: RefMut<'_, TestServer>,
                               csrf_token: HeaderValue,
                               request_cookie: Cookie<'_>,
                               id: &i32) -> Value {

        let query = format!(r#"
            {{
                "query": "
                    mutation DestroyAProduct($productId: Int!) {{
                        destroyProduct(productId: $productId)
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
            request
                .send_body(query)
                .await
                .unwrap();
        assert!(response.status().is_success());

        let bytes = response.body().await.unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        serde_json::from_str(body).unwrap()
    }

    async fn search_products(srv: RefMut<'_, TestServer>,
                             csrf_token: HeaderValue,
                             request_cookie: Cookie<'_>,
                             data_for_searching: Value) {

        let query = format!(r#"
            {{
                "query": "
                    query ListProduct($search: String!, $limit: Int!, $rank: Float!) {{
                        listProduct(search: $search, limit: $limit, rank: $rank) {{
                            data {{
                                product {{
                                    id
                                    name
                                    stock
                                    cost
                                    description
                                }}
                                priceProducts {{
                                    priceProduct {{
                                        amount
                                    }}
                                    price {{
                                        name
                                    }}
                                }}
                            }}
                        }}
                    }}
                ",
                "variables": {{
                    "search": "hat",
                    "limit": 10,
                    "rank": 1.0
                }}
            }}
        "#).replace("\n", "");

        let request = srv
                          .post("/graphql")
                          .header(header::CONTENT_TYPE, "application/json")
                          .header("x-csrf-token", csrf_token.to_str().unwrap())
                          .cookie(request_cookie)
                          .timeout(std_duration::from_secs(600));

        let mut response =
            request
                .send_body(query)
                .await
                .unwrap();
        assert!(response.status().is_success());

        let bytes = response.body().await.unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        let response_sales: Value = serde_json::from_str(body).unwrap();
        assert_eq!(data_for_searching, response_sales);
    }

    async fn create_a_price(srv: RefMut<'_, TestServer>,
                            csrf_token: HeaderValue,
                            request_cookie: Cookie<'_>,
                            price: &FormPrice) -> Value {

        let request = srv
                        .post("/graphql")
                        .header(header::CONTENT_TYPE, "application/json")
                        .header("x-csrf-token", csrf_token.to_str().unwrap())
                        .cookie(request_cookie)
                        .timeout(std_duration::from_secs(600));

        let query =
            format!(
            r#"
            {{
                "query": "
                    mutation createPrice($form: FormPrice!) {{
                            createPrice(form: $form) {{
                                id
                                name
                                userId
                            }}
                    }}
                ",
                "variables": {{
                    "form": {{
                        "name": "{}"
                    }}
                }}
            }}"#,
            price.clone().name.unwrap())
            .replace("\n", "");

        let mut response =
            request
                .send_body(query)
                .await
                .unwrap();

        assert!(response.status().is_success());

        let bytes = response.body().await.unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        serde_json::from_str(body).unwrap()
    }
}