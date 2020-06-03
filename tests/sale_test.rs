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
    use chrono::Local;
    use chrono::NaiveDate;
    use csrf_token::CsrfTokenGenerator;
    use http::header::HeaderValue;
    use actix_http::cookie::Cookie;
    use actix_http::httpmessage::HttpMessage;

    use actix_service::map_config;
    use actix_web::dev::AppConfig;
    use serde_json::{json, Value};
    use std::str;
    use std::time::Duration as std_duration;
    use crate::common::db_connection::establish_connection;
    use std::cell::{RefCell, RefMut};

    use ::mystore_lib::models::product::{Product, NewProduct, FullProduct};
    use ::mystore_lib::models::user::{ NewUser, User };
    use ::mystore_lib::graphql::schema::create_schema;
    use ::mystore_lib::graphql::{graphql, graphiql};
    use ::mystore_lib::models::sale;
    use ::mystore_lib::models::sale_state::SaleState;
    use ::mystore_lib::models::sale_product::FormSaleProduct;
    use ::mystore_lib::models::price::NewPriceProductsToUpdate;

    #[actix_rt::test]
    async fn test() {

        let user = create_user();

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

        let new_shoe = NewProduct {
            id: None,
            name: Some("Shoe".to_string()),
            stock: Some(10.4),
            cost: Some(1892),
            description: Some("not just your regular shoes, this one will make you jump".to_string()),
            user_id: Some(user.id)
        };

        let new_hat = NewProduct {
            id: None,
            name: Some("Hat".to_string()),
            stock: Some(15.0),
            cost: Some(2045),
            description: Some("Just a regular hat".to_string()),
            user_id: Some(user.id)
        };

        let _new_pants = NewProduct {
            id: None,
            name: Some("Pants".to_string()),
            stock: Some(25.0),
            cost: Some(3025),
            description: Some("beautiful black pants that will make you look thin".to_string()),
            user_id: Some(user.id)
        };

        let shoe = create_product(user.id, new_shoe).product;
        let hat = create_product(user.id, new_hat).product;

        let new_sale = sale::Form {
            id: None,
            user_id: None,
            sale_date: Some(NaiveDate::from_ymd(2019, 11, 12)),
            total: Some(123.98),
            bill_number: None,
            state: Some(SaleState::Draft)
        };

        let new_sale_product = FormSaleProduct {
            id: None,
            product_id: Some(shoe.id),
            sale_id: None,
            amount: Some(8.0),
            discount: Some(0),
            tax: Some(12),
            price: Some(20),
            total: Some(28.0)
        };

        let response_sale = 
            create_a_sale(srv.borrow_mut(), 
                        csrf_token.clone(),
                        request_cookie.clone(),
                        &new_sale,
                        vec![&new_sale_product]).await;

        let sale = response_sale.get("data").unwrap().get("createSale").unwrap();
        let sale_id: i32 = serde_json::from_value(sale.get("sale").unwrap().get("id").unwrap().clone()).unwrap();

        show_a_sale(srv.borrow_mut(), 
                    csrf_token.clone(),
                    request_cookie.clone(),
                    &sale_id,
                    sale).await;

        let new_sale_to_update = sale::Form {
            id: Some(sale_id),
            user_id: None,
            sale_date: Some(NaiveDate::from_ymd(2019, 11, 10)),
            total: Some(123.98),
            bill_number: None,
            state: Some(SaleState::Draft)
        };

        let new_sale_product_hat = FormSaleProduct {
            id: None,
            product_id: Some(hat.id),
            sale_id: None,
            amount: Some(5.0),
            discount: Some(0),
            tax: Some(12),
            price: Some(30),
            total: Some(150.0)
        };

        let response_sale = 
            update_a_sale(srv.borrow_mut(), 
                        csrf_token.clone(),
                        request_cookie.clone(),
                        &new_sale_to_update,
                        vec![&new_sale_product_hat]).await;

        let sale = response_sale.get("data").unwrap().get("updateSale").unwrap();
        assert_eq!(sale.get("sale").unwrap().get("saleDate").unwrap(), "2019-11-10");

        let data_to_compare = json!({
            "data": {
                "listSale": {
                    "data": [{
                        "sale": {
                            "id": sale_id,
                            "saleDate": "2019-11-10",
                            "total": 123.98,
                        },
                        "saleProducts": [{
                            "product":
                            {
                                "name": "Hat",
                            },
                            "saleProduct":
                            {
                                "amount": 5.0,
                                "price": 30,
                            }
                        }]
                    }]
                }
            }
        });

        search_sales(srv.borrow_mut(), csrf_token.clone(), request_cookie.clone(), data_to_compare).await;

        let response_state = cancel_a_sale(srv.borrow_mut(), csrf_token.clone(), request_cookie.clone(), sale_id).await;
        let errors: Vec<Value> =
            serde_json::from_value(
                response_state
                    .get("errors")
                    .unwrap()
                    .clone()
            ).unwrap();

        let state_result: String = 
            serde_json::from_value(
                errors
                    .first()
                    .unwrap()
                    .get("message")
                    .unwrap()
                    .clone()
                ).unwrap();
        assert_eq!(state_result, "You can\'t Cancel from Draft state".to_string());

        let response_state =
            approve_a_sale(srv.borrow_mut(),
                           csrf_token.clone(),
                           request_cookie.clone(),
                           sale_id).await;
        let state_result: bool = 
            serde_json::from_value(
                response_state
                    .get("data")
                    .unwrap()
                    .get("approveSale")
                    .unwrap()
                    .clone()
                ).unwrap();
        assert!(state_result);

        let response_sale_destroyed = 
            destroy_a_sale(srv.borrow_mut(), 
                           csrf_token.clone(),
                           request_cookie.clone(),
                           &sale_id).await;
        
        let destroyed: bool =
            serde_json::from_value(
                response_sale_destroyed
                    .get("data")
                    .unwrap()
                    .get("destroySale")
                    .unwrap()
                    .clone()
            ).unwrap();
        assert!(!destroyed);

        let response_sale = 
            create_a_sale(srv.borrow_mut(), 
                        csrf_token.clone(),
                        request_cookie.clone(),
                        &new_sale,
                        vec![&new_sale_product]).await;

        let sale = response_sale.get("data").unwrap().get("createSale").unwrap();
        let sale_id: i32 = serde_json::from_value(sale.get("sale").unwrap().get("id").unwrap().clone()).unwrap();

        let response_sale_destroyed = 
            destroy_a_sale(srv.borrow_mut(), 
                           csrf_token.clone(),
                           request_cookie.clone(),
                           &sale_id).await;
        
        let destroyed: bool =
            serde_json::from_value(
                response_sale_destroyed
                    .get("data")
                    .unwrap()
                    .get("destroySale")
                    .unwrap()
                    .clone()
            ).unwrap();
        assert!(destroyed);
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

    fn create_product(user_id: i32, new_product: NewProduct) -> FullProduct {
        use std::sync::Arc;
        use ::mystore_lib::models::Context;

        let connection = establish_connection();
        let pg_pool = connection.get().unwrap();
        let context = Context {
            user_id,
            conn: Arc::new(pg_pool)
        };
        Product::create(&context, new_product, NewPriceProductsToUpdate{data: vec![]}).unwrap()
    }

    async fn create_a_sale(srv: RefMut<'_, TestServer>,
                            csrf_token: HeaderValue,
                            request_cookie: Cookie<'_>,
                            new_sale: &sale::Form,
                            new_sale_products: Vec<&FormSaleProduct>) -> Value {

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
                    mutation CreateSale($form: Form!, $paramFormSaleProducts: FormSaleProducts!) {{
                            createSale(form: $form, paramFormSaleProducts: $paramFormSaleProducts) {{
                                sale {{
                                    id
                                    userId
                                    saleDate
                                    total
                                    state
                                }}
                                saleProducts {{
                                    product {{
                                        name
                                    }}
                                    saleProduct {{
                                        id
                                        productId
                                        amount
                                        discount
                                        tax
                                        price
                                        total
                                    }}
                                }}
                            }}
                    }}
                ",
                "variables": {{
                    "form": {{
                        "saleDate": "{}",
                        "total": {}
                    }},
                    "paramFormSaleProducts": {{
                        "data":
                            [{{
                                "product": {{ }},
                                "saleProduct": {{
                                    "amount": {},
                                    "discount": {},
                                    "price": {},
                                    "productId": {},
                                    "tax": {},
                                    "total": {}
                                }}
                            }}]
                    }}
                }}
            }}"#,
            new_sale.sale_date.unwrap(), new_sale.total.unwrap(),
            new_sale_products.get(0).unwrap().amount.unwrap(),
            new_sale_products.get(0).unwrap().discount.unwrap(),
            new_sale_products.get(0).unwrap().price.unwrap(),
            new_sale_products.get(0).unwrap().product_id.unwrap(),
            new_sale_products.get(0).unwrap().tax.unwrap(),
            new_sale_products.get(0).unwrap().total.unwrap())
            .replace("\n", "");

        let mut response =
            request
                .send_body(query)
                .await
                .unwrap();

        assert!(response.status().is_success());

        let bytes = response.body().await.unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        println!("{:#?}", &body);
        serde_json::from_str(body).unwrap()
    }

    async fn show_a_sale(srv: RefMut<'_, TestServer>,
                         csrf_token: HeaderValue,
                         request_cookie: Cookie<'_>,
                         id: &i32,
                         expected_sale: &Value) {

        let query = format!(r#"
            {{
                "query": "
                    query ShowASale($saleId: Int!) {{
                        sale(saleId: $saleId) {{
                            sale {{
                                id
                                userId
                                saleDate
                                total
                                state
                            }}
                            saleProducts {{
                                product {{ name }}
                                saleProduct {{
                                    id
                                    productId
                                    amount
                                    discount
                                    tax
                                    price
                                    total
                                }}
                            }}
                        }}
                    }}
                ",
                "variables": {{
                    "saleId": {}
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
        let response_sale: Value = serde_json::from_str(body).unwrap();
        let sale = response_sale.get("data").unwrap().get("sale").unwrap();
        assert_eq!(sale, expected_sale);
    }

    async fn update_a_sale(srv: RefMut<'_, TestServer>,
                           csrf_token: HeaderValue,
                           request_cookie: Cookie<'_>,
                           changes_to_sale: &sale::Form,
                           changes_to_sale_products: Vec<&FormSaleProduct>) -> Value {

        let query = 
            format!(
            r#"
            {{
                "query": "
                    mutation UpdateSale($form: Form!, $paramSaleProducts: FormSaleProducts!) {{
                            updateSale(form: $form, paramSaleProducts: $paramSaleProducts) {{
                                sale {{
                                    id
                                    saleDate
                                    total
                                }}
                                saleProducts {{
                                    product {{ name }}
                                    saleProduct {{
                                        id
                                        productId
                                        amount
                                        discount
                                        tax
                                        price
                                        total
                                    }}
                                }}
                            }}
                    }}
                ",
                "variables": {{
                    "form": {{
                        "id": {},
                        "saleDate": "{}",
                        "total": {}
                    }},
                    "paramSaleProducts": {{
                        "data":
                            [{{
                                "product": {{}},
                                "saleProduct": 
                                {{
                                    "amount": {},
                                    "discount": {},
                                    "price": {},
                                    "productId": {},
                                    "tax": {},
                                    "total": {}
                                }}
                            }}]
                    }}
                }}
            }}"#,
            changes_to_sale.id.unwrap(),
            changes_to_sale.sale_date.unwrap(), changes_to_sale.total.unwrap(),
            changes_to_sale_products.get(0).unwrap().amount.unwrap(),
            changes_to_sale_products.get(0).unwrap().discount.unwrap(),
            changes_to_sale_products.get(0).unwrap().price.unwrap(),
            changes_to_sale_products.get(0).unwrap().product_id.unwrap(),
            changes_to_sale_products.get(0).unwrap().tax.unwrap(),
            changes_to_sale_products.get(0).unwrap().total.unwrap())
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

    async fn approve_a_sale(srv: RefMut<'_, TestServer>,
                            csrf_token: HeaderValue,
                            request_cookie: Cookie<'_>,
                            id: i32) -> Value {

        let query = format!(r#"
            {{
                "query": "
                    mutation ApproveSale($saleId: Int!) {{
                        approveSale(saleId: $saleId)
                    }}
                ",
                "variables": {{
                    "saleId": {}
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

    async fn cancel_a_sale(srv: RefMut<'_, TestServer>,
                           csrf_token: HeaderValue,
                           request_cookie: Cookie<'_>,
                           id: i32) -> Value {

        let query = format!(r#"
            {{
                "query": "
                    mutation CancelSale($saleId: Int!) {{
                        cancelSale(saleId: $saleId)
                    }}
                ",
                "variables": {{
                    "saleId": {}
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

    async fn destroy_a_sale(srv: RefMut<'_, TestServer>,
                            csrf_token: HeaderValue,
                            request_cookie: Cookie<'_>,
                            id: &i32) -> Value {
        let query = format!(r#"
            {{
                "query": "
                    mutation DestroyASale($saleId: Int!) {{
                        destroySale(saleId: $saleId)
                    }}
                ",
                "variables": {{
                    "saleId": {}
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

    async fn search_sales(srv: RefMut<'_, TestServer>,
                          csrf_token: HeaderValue,
                          request_cookie: Cookie<'_>,
                          data_to_compare: Value) {

        let query = format!(r#"
            {{
                "query": "
                    query ListSale($search: Form!, $limit: Int!) {{
                        listSale(search: $search, limit: $limit) {{
                            data {{
                                sale {{
                                    id
                                    saleDate
                                    total
                                }}
                                saleProducts {{
                                    product {{
                                        name
                                    }}
                                    saleProduct {{
                                        amount
                                        price
                                    }}
                                }}
                            }}
                        }}
                    }}
                ",
                "variables": {{
                    "search": {{
                        "saleDate": "2019-11-10"
                    }},
                    "limit": 10
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
        assert_eq!(data_to_compare, response_sales);
    }
}