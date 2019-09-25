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
    use chrono::Local;
    use chrono::NaiveDate;
    use csrf_token::CsrfTokenGenerator;
    use actix_http::httpmessage::HttpMessage;
    use http::header::HeaderValue;
    use actix_http::cookie::Cookie;

    use serde_json::{ json, Value };
    use std::str;
    use std::time::Duration as std_duration;
    use crate::common::db_connection::establish_connection;
    use std::cell::{ RefCell, RefMut };

    use ::mystore_lib::models::product::{ Product, NewProduct, ProductList };
    use ::mystore_lib::models::user::{ NewUser, User };
    use ::mystore_lib::models::sale::create_schema;
    use ::mystore_lib::graphql::{graphql, graphiql};
    use ::mystore_lib::models::sale::{ ListSale, NewSale };
    use ::mystore_lib::models::sale_state::SaleState;
    use ::mystore_lib::models::sale_product::{ NewSaleProduct, NewSaleProducts };

    #[test]
    fn test() {

        let user = create_user();

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
                        web::resource("/graphiql").route(web::get().to(graphiql))
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

        let new_pants = NewProduct {
            id: None,
            name: Some("Pants".to_string()),
            stock: Some(25.0),
            cost: Some(3025),
            description: Some("beautiful black pants that will make you look thin".to_string()),
            user_id: Some(user.id)
        };

        let shoe = create_product(user.id, new_shoe);
        let hat = create_product(user.id, new_hat);

        let new_sale = NewSale {
            id: None,
            user_id: None,
            sale_date: Some(NaiveDate::from_ymd(2019, 11, 12)),
            total: Some(123.98),
            bill_number: None,
            state: Some(SaleState::Draft)
        };

        let new_sale_product = NewSaleProduct {
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
                        vec![&new_sale_product]);

        let sale = response_sale.get("data").unwrap().get("createSale").unwrap();
        let sale_id: i32 = serde_json::from_value(sale.get("sale").unwrap().get("id").unwrap().clone()).unwrap();

        show_a_sale(srv.borrow_mut(), 
                    csrf_token.clone(),
                    request_cookie.clone(),
                    &sale_id,
                    sale);

        let new_sale_to_update = NewSale {
            id: Some(sale_id),
            user_id: None,
            sale_date: Some(NaiveDate::from_ymd(2019, 11, 10)),
            total: Some(123.98),
            bill_number: None,
            state: Some(SaleState::Draft)
        };

        let new_sale_product_hat = NewSaleProduct {
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
                        vec![&new_sale_product_hat]);

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

        search_sales(srv.borrow_mut(), csrf_token.clone(), request_cookie.clone(), data_to_compare);

        let response_sale_id_destroyed = 
            destroy_a_sale(srv.borrow_mut(), 
                           csrf_token.clone(),
                           request_cookie.clone(),
                           &sale_id);
        
        let sale_id_destroyed: i32 =
         serde_json::from_value(
             response_sale_id_destroyed
                 .get("data")
                 .unwrap()
                 .get("destroySale")
                 .unwrap()
                 .clone()
         ).unwrap();
        assert_eq!(sale_id, sale_id_destroyed);
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

    fn create_product(user_id: i32, new_product: NewProduct) -> Product {
        let connection = establish_connection();
        let pg_pool = connection.get().unwrap();
        new_product.create(user_id, vec![], &pg_pool).unwrap().0
    }

    fn create_a_sale(mut srv: RefMut<TestServerRuntime>,
                            csrf_token: HeaderValue,
                            request_cookie: Cookie,
                            new_sale: &NewSale,
                            new_sale_products: Vec<&NewSaleProduct>) -> Value {

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
                    mutation CreateSale($paramNewSale: NewSale!, $paramNewSaleProducts: NewSaleProducts!) {{
                            createSale(paramNewSale: $paramNewSale, paramNewSaleProducts: $paramNewSaleProducts) {{
                                sale {{
                                    id
                                    userId
                                    saleDate
                                    total
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
                    "paramNewSale": {{
                        "saleDate": "{}",
                        "total": {}
                    }},
                    "paramNewSaleProducts": {{
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
            srv
                .block_on(request.send_body(query))
                .unwrap();

        assert!(response.status().is_success());

        let bytes = srv.block_on(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        serde_json::from_str(body).unwrap()
    }

    fn show_a_sale(mut srv: RefMut<TestServerRuntime>,
                       csrf_token: HeaderValue,
                       request_cookie: Cookie,
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
        let response_sale: Value = serde_json::from_str(body).unwrap();
        let sale = response_sale.get("data").unwrap().get("sale").unwrap();
        assert_eq!(sale, expected_sale);
    }

    fn update_a_sale(mut srv: RefMut<TestServerRuntime>,
                         csrf_token: HeaderValue,
                         request_cookie: Cookie,
                         changes_to_sale: &NewSale,
                         changes_to_sale_products: Vec<&NewSaleProduct>) -> Value {

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
                    mutation UpdateSale($paramSale: NewSale!, $paramSaleProducts: NewSaleProducts!) {{
                            updateSale(paramSale: $paramSale, paramSaleProducts: $paramSaleProducts) {{
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
                    "paramSale": {{
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

        let mut response =
            srv
                .block_on(request.send_body(query))
                .unwrap();

        assert!(response.status().is_success());

        let bytes = srv.block_on(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        serde_json::from_str(body).unwrap()
    }

    fn destroy_a_sale(mut srv: RefMut<TestServerRuntime>,
                          csrf_token: HeaderValue,
                          request_cookie: Cookie,
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
            srv
                .block_on(request.send_body(query))
                .unwrap();
        assert!(response.status().is_success());

        let bytes = srv.block_on(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        serde_json::from_str(body).unwrap()
    }

    fn search_sales(mut srv: RefMut<TestServerRuntime>,
                        csrf_token: HeaderValue,
                        request_cookie: Cookie,
                        data_to_compare: Value) {

        let query = format!(r#"
            {{
                "query": "
                    query ListSale($search: NewSale!, $limit: Int!) {{
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
            srv
                .block_on(request.send_body(query))
                .unwrap();
        assert!(response.status().is_success());

        let bytes = srv.block_on(response.body()).unwrap();
        let body = str::from_utf8(&bytes).unwrap();
        let response_sales: Value = serde_json::from_str(body).unwrap();
        dbg!(&response_sales);
        assert_eq!(data_to_compare, response_sales);
    }
}