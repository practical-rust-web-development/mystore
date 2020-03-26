use actix::prelude::Future;
use actix_web::{ web, HttpResponse };

use crate::models::price::PriceList;
use crate::handlers::LoggedUser;
use crate::db_connection::PgPool;
use crate::db_connection::PgPooledConnection;

function_handler!(
    index () -> (|user: LoggedUser, pg_pool: PgPooledConnection| {
        PriceList::list(user.id, &pg_pool)
    })
);

use crate::models::price::NewPrice;

function_handler!(
    create (new_price: web::Json<NewPrice>)
        -> (|user: LoggedUser, pg_pool: PgPooledConnection| {
            new_price.create(user.id, &pg_pool)
        })
);

use crate::models::price::Price;

function_handler!(
    show (id: web::Path<i32>) -> (|user: LoggedUser, pg_pool: PgPooledConnection| {
        Price::find(*id, user.id, &pg_pool)
    })
);

function_handler!(
    destroy (id: web::Path<i32>) -> (|user: LoggedUser, pg_pool: PgPooledConnection| {
        Price::destroy(*id, user.id, &pg_pool)
    })
);

function_handler!(
    update (id: web::Path<i32>, new_price: web::Json<NewPrice>)
     -> (|user: LoggedUser, pg_pool: PgPooledConnection| {
        Price::update(user.id, *id, new_price.clone(), &pg_pool)
    })
);