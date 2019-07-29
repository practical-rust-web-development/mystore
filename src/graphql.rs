use std::sync::Arc;
use std::ops::Deref;
use actix_web::{web, Error, HttpResponse};
use actix::prelude::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

use crate::models::sale::{create_schema, Schema, Context, create_context};
use crate::handlers::LoggedUser;
use crate::db_connection::PgPool;
use crate::serde::ser::Error as SerdeError;

pub fn graphiql() -> HttpResponse {
    let html = graphiql_source("http://127.0.0.1:8080/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

pub fn graphql(
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
    user: LoggedUser,
    pool: web::Data<PgPool>
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        let pg_pool = pool
            .get()
            .map_err(|e| {
                serde_json::Error::custom(e)
            })?;
        
        let ctx = create_context(user.id, pg_pool);

        let res = data.execute(&st, &ctx);
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .map_err(Error::from)
    .and_then(|user| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(user))
    })
}