use juniper::FieldResult;
use crate::models::Context;
use crate::models::sale::{FormSale, Sale, FullSale};
use crate::models::sale_product::FormSaleProducts;
use crate::models::price::FormPriceProductsToUpdate;
use crate::models::product::{FullProduct, Product, FormProduct};
use crate::models::sale_state::Event;
use crate::models::price::{FormPrice, Price};

pub struct Mutation;

#[juniper::object(
    Context = Context,
)]
impl Mutation {
    fn createSale(
        context: &Context,
        form: FormSale,
        form_sale_products: FormSaleProducts,
    ) -> FieldResult<FullSale> {
        Sale::create(context, form, form_sale_products)
    }

    fn updateSale(
        context: &Context,
        form: FormSale,
        form_sale_products: FormSaleProducts,
    ) -> FieldResult<FullSale> {
        Sale::update(context, form, form_sale_products)
    }

    fn approveSale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        Sale::set_state(context, sale_id, Event::Approve)
    }

    fn cancelSale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        //TODO: perform credit note or debit note
        Sale::set_state(context, sale_id, Event::Cancel)
    }

    fn paySale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        //TODO: perform collection
        Sale::set_state(context, sale_id, Event::Pay)
    }

    fn partiallyPaySale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        //TODO: perform collection
        Sale::set_state(context, sale_id, Event::PartiallyPay)
    }

    fn destroySale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        Sale::destroy(context, sale_id)
    }

    fn createProduct(
        context: &Context,
        form: FormProduct,
        form_price_products: FormPriceProductsToUpdate,
    ) -> FieldResult<FullProduct> {
        Product::create(context, form, form_price_products)
    }

    fn updateProduct(
        context: &Context,
        form: FormProduct,
        form_price_products: FormPriceProductsToUpdate,
    ) -> FieldResult<FullProduct> {
        Product::update(context, form, form_price_products)
    }

    fn destroyProduct(context: &Context, product_id: i32) -> FieldResult<bool> {
        Product::destroy(context, product_id)
    }

    fn createPrice(context: &Context, form: FormPrice) -> FieldResult<Price> {
        Price::create(context, form)
    }

    fn updatePrice(context: &Context, form: FormPrice) -> FieldResult<Price> {
        Price::update(context, form)
    }

    fn destroyPrice(context: &Context, price_id: i32) -> FieldResult<bool> {
        Price::destroy(context, price_id)
    }
}
