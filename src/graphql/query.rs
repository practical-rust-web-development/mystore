use juniper::FieldResult;
use crate::models::Context;
use crate::models::sale::{Sale, NewSale, ListSale, FullSale};
use crate::models::product::{Product, ListProduct, FullProduct};
use crate::models::price::{PriceList, Price};

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

    fn listProduct(context: &Context, search: String, limit: i32, rank: f64) -> FieldResult<ListProduct> {
        Product::list_product(context, search, limit, rank)
    }

    fn product(context: &Context, product_id: i32) -> FieldResult<FullProduct> {
        Product::product(context, product_id)
    }

    fn ListPrice(context: &Context) -> FieldResult<PriceList> {
        PriceList::list(context)
    }

    fn price(context: &Context, price_id: i32) -> FieldResult<Price> {
        Price::find(context, price_id)
    }

}