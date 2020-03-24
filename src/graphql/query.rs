use juniper::FieldResult;
use crate::models::Context;
use crate::models::sale::{Sale, NewSale, ListSale, FullSale};

pub struct Query;

#[juniper::object(
    Context = Context,
)]
impl Query {
    fn listSale(context: &Context, search: Option<NewSale>, limit: i32) -> FieldResult<ListSale> {
        Sale::list_sale(context, search, limit)
    }

    fn sale(context: &Context, sale_id: i32) -> FieldResult<FullSale> {
        Sale::sale(context, sale_id)
    }
}