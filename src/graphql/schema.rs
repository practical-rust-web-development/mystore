use crate::graphql::mutation::Mutation;
use crate::graphql::query::Query;

pub type Schema = juniper::RootNode<'static, Query, Mutation>;

pub fn create_schema() -> Schema {
    Schema::new(Query {}, Mutation {})
}
