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