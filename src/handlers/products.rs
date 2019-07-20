use actix::prelude::Future;
use actix_web::{ web, HttpResponse };

use crate::models::product::ProductList;
use crate::handlers::LoggedUser;
use crate::db_connection::PgPool;

#[derive(Deserialize)]
pub struct ProductSearch{ 
    pub search: String
}

#[derive(Deserialize)]
pub struct ProductPagination {
    pub rank: f64
}

use serde::Serialize;

use crate::db_connection::PgPooledConnection;

function_handler!(
    index (product_search: web::Query<ProductSearch>, pagination: web::Query<ProductPagination>)
     -> (|user: LoggedUser, pg_pool: PgPooledConnection| {
            let search = &product_search.search;
            ProductList::list(user.id, search, pagination.rank, &pg_pool)
        })
);

use crate::models::product::NewProduct;
use crate::models::price::PriceProductToUpdate;

#[derive(Serialize, Deserialize, Clone)]
pub struct ProductWithPrices {
    pub product: NewProduct,
    pub prices: Vec<PriceProductToUpdate>
}

function_handler!(
    create (new_product_with_prices: web::Json<ProductWithPrices>)
     -> (|user: LoggedUser, pg_pool: PgPooledConnection| {
            new_product_with_prices
                .product
                .create(user.id, new_product_with_prices.clone().prices, &pg_pool)
        })
);

use crate::models::product::Product;

function_handler!(
    show (id: web::Path<i32>) -> (|user: LoggedUser, pg_pool: PgPooledConnection| {
        Product::find(&id, user.id, &pg_pool)
    })
);

function_handler!(
    destroy (id: web::Path<i32>) -> (|user: LoggedUser, pg_pool: PgPooledConnection| {
        Product::destroy(&id, user.id, &pg_pool)
    })
);

function_handler!(
    update (id: web::Path<i32>, new_product: web::Json<ProductWithPrices>) 
     -> (|user: LoggedUser, pg_pool: PgPooledConnection| {
        let product_id = *id;
        let product = new_product.clone();
        Product::update(product_id, user.id, product.product, product.prices, &pg_pool)
    })
);