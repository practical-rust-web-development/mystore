pub mod query;
pub mod mutation;
pub mod schema;

use std::sync::Arc;
use actix_web::{web, Error, HttpResponse, post, get};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use schema::Schema;

use crate::models::create_context;
use crate::handlers::LoggedUser;
use crate::db_connection::PgPool;
use crate::serde::ser::Error as SerdeError;

#[get("/graphiql")]
pub async fn graphiql() -> HttpResponse {
    let html = graphiql_source("http://127.0.0.1:8080/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

#[post("/graphql")]
pub async fn graphql(
    st: web::Data<Arc<Schema>>,
    data: web::Json<GraphQLRequest>,
    user: LoggedUser,
    pool: web::Data<PgPool>
) -> Result<HttpResponse, Error> {
    let user =
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
        .await?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(user))
}