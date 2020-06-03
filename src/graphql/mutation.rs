use juniper::FieldResult;
use crate::models::Context;
use crate::models::sale;
use crate::models::sale_product::NewSaleProducts;
use crate::models::price::NewPriceProductsToUpdate;
use crate::models::product::{FullProduct, Product, NewProduct};
use crate::models::sale_state::Event;
use crate::models::price::{NewPrice, Price};

pub struct Mutation;

#[juniper::object(
    Context = Context,
)]
impl Mutation {
    fn createSale(
        context: &Context,
        form: sale::Form,
        param_new_sale_products: NewSaleProducts,
    ) -> FieldResult<sale::FullSale> {
        sale::Sale::create(context, form, param_new_sale_products)
    }

    fn approveSale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        sale::Sale::set_state(context, sale_id, Event::Approve)
    }

    fn cancelSale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        //TODO: perform credit note or debit note
        sale::Sale::set_state(context, sale_id, Event::Cancel)
    }

    fn paySale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        //TODO: perform collection
        sale::Sale::set_state(context, sale_id, Event::Pay)
    }

    fn partiallyPaySale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        //TODO: perform collection
        sale::Sale::set_state(context, sale_id, Event::PartiallyPay)
    }

    fn updateSale(
        context: &Context,
        form: sale::Form,
        param_sale_products: NewSaleProducts,
    ) -> FieldResult<sale::FullSale> {
        sale::Sale::update(context, form, param_sale_products)
    }

    fn destroySale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        sale::Sale::destroy(context, sale_id)
    }

    fn createProduct(
        context: &Context,
        param_new_product: NewProduct,
        param_new_price_products: NewPriceProductsToUpdate,
    ) -> FieldResult<FullProduct> {
        Product::create(context, param_new_product, param_new_price_products)
    }

    fn updateProduct(
        context: &Context,
        param_product: NewProduct,
        param_price_products: NewPriceProductsToUpdate,
    ) -> FieldResult<FullProduct> {
        Product::update(context, param_product, param_price_products)
    }

    fn destroyProduct(context: &Context, product_id: i32) -> FieldResult<bool> {
        Product::destroy(context, product_id)
    }

    fn createPrice(context: &Context, new_price: NewPrice) -> FieldResult<Price> {
        Price::create(context, new_price)
    }

    fn updatePrice(context: &Context, edit_price: NewPrice) -> FieldResult<Price> {
        Price::update(context, edit_price)
    }

    fn destroyPrice(context: &Context, price_id: i32) -> FieldResult<bool> {
        Price::destroy(context, price_id)
    }
}
