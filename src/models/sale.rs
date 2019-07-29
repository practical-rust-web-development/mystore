use diesel::PgConnection;
use chrono::NaiveDateTime;
use juniper::{FieldResult};
use crate::schema;
use crate::schema::sales;
use crate::schema::sales::dsl::*;
use crate::db_connection::{ PgPool, PgPooledConnection };

#[derive(Identifiable, Queryable, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[table_name="sales"]
#[derive(juniper::GraphQLObject)]
#[graphql(description="Sale Bill")]
pub struct Sale {
    pub id: i32,
    pub user_id: i32,
    pub sale_date: NaiveDateTime,
    pub total: f64
}

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq)]
#[table_name="sales"]
#[derive(juniper::GraphQLInputObject)]
#[graphql(description="Sale Bill")]
pub struct NewSale {
    pub sale_date: NaiveDateTime,
    pub total: f64
}

use std::sync::Arc;

pub struct Context {
    pub user_id: i32,
    pub conn: Arc<PgPooledConnection>,
}

impl juniper::Context for Context {}

pub struct Query;

#[juniper::object(
    Context = Context,
)]
impl Query {

    fn sale(context: &Context, sale_id: String) -> FieldResult<Sale> {
        use diesel::{ ExpressionMethods, QueryDsl, RunQueryDsl };

        let conn: &PgConnection = &context.conn;
        let sale: Sale =
            schema::sales::table
                .filter(user_id.eq(context.user_id))
                .find(sale_id.parse::<i32>().unwrap())
                .first::<Sale>(conn)?;
        Ok(sale)
    }
}

pub struct Mutation;

#[juniper::object(
    Context = Context,
)]
impl Mutation {
    fn createSale(context: &Context, new_sale: NewSale) -> FieldResult<Sale> {
        use crate::schema::sales::dsl::*;
        use diesel::RunQueryDsl;

        let conn: &PgConnection = &context.conn;
        let sale = 
            diesel::insert_into(schema::sales::table)
                .values(new_sale)
                .returning((id, user_id, sale_date, total))
                .get_result::<Sale>(conn)?;

        Ok(sale)
    }
}

pub type Schema = juniper::RootNode<'static, Query, Mutation>;

pub fn create_schema() -> Schema {
    Schema::new(Query {}, Mutation {})
}

pub fn create_context(logged_user_id: i32, pg_pool: PgPooledConnection) -> Context {
    Context { user_id: logged_user_id, conn: Arc::new(pg_pool)}
}