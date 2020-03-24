use crate::graphql::query::Query;
use crate::graphql::mutation::Mutation;

pub type Schema = juniper::RootNode<'static, Query, Mutation>;

pub fn create_schema() -> Schema {
    Schema::new(Query {}, Mutation {})
}