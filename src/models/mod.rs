pub mod price;
pub mod user;
pub mod product;
pub mod sale;
pub mod sale_product;
pub mod sale_state;

pub fn show_query<T>(query: &T)
where
    T: diesel::query_builder::QueryFragment<diesel::pg::Pg>,
{
    dbg!(diesel::debug_query::<diesel::pg::Pg, _>(&query));
}

use std::sync::Arc;
use crate::db_connection::PgPooledConnection;

pub struct Context {
    pub user_id: i32,
    pub conn: Arc<PgPooledConnection>,
}

impl juniper::Context for Context {}

pub fn create_context(logged_user_id: i32, pg_pool: PgPooledConnection) -> Context {
    Context {
        user_id: logged_user_id,
        conn: Arc::new(pg_pool),
    }
}