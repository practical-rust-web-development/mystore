use juniper::FieldResult;
use crate::schema::prices;
use crate::schema::prices::dsl::*;
use crate::schema::prices_products;
use crate::models::product::Product;
use crate::models::Context;

#[derive(Serialize, Deserialize, Clone, juniper::GraphQLObject)]
pub struct PriceList {
    pub data: Vec<Price>
}

#[derive(Identifiable, Queryable, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[table_name="prices"]
#[derive(juniper::GraphQLObject)]
pub struct Price {
    pub id: i32,
    pub name: String,
    pub user_id: i32
}

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq, juniper::GraphQLInputObject)]
#[table_name="prices"]
pub struct NewPrice {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub user_id: Option<i32>
}

#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[belongs_to(Price)]
#[belongs_to(Product)]
#[table_name="prices_products"]
#[derive(juniper::GraphQLObject)]
pub struct PriceProduct {
    pub id: i32,
    pub price_id: i32,
    pub product_id: i32,
    pub user_id: i32,
    pub amount: Option<i32>
}

#[derive(juniper::GraphQLObject)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullPriceProduct {
    pub price_product: PriceProduct,
    pub price: Price
}

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq)]
#[table_name="prices_products"]
#[derive(juniper::GraphQLInputObject)]
pub struct NewPriceProduct {
    pub id: Option<i32>,
    pub price_id: i32,
    pub product_id: Option<i32>,
    pub user_id: Option<i32>,
    pub amount: Option<i32>
}

#[derive(Clone, juniper::GraphQLInputObject)]
pub struct NewPriceProductsToUpdate{ pub data: Vec<PriceProductToUpdate> }

#[derive(Serialize, Deserialize, Clone)]
#[derive(juniper::GraphQLInputObject)]
pub struct PriceProductToUpdate {
    pub price_product: NewPriceProduct,
    pub to_delete: bool
}

use diesel::PgConnection;

impl PriceProductToUpdate {
    pub fn batch_update(context: &Context, records: NewPriceProductsToUpdate, param_product_id: i32) ->
        Result<Vec<FullPriceProduct>, diesel::result::Error> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;
            use diesel::Connection;
            use itertools::Itertools;

            let connection: &PgConnection = &context.conn;
            let param_user_id = context.user_id;

            connection.transaction(|| {
                let mut records_to_keep = vec![];
                for price_product_to_update in records.data {

                    if price_product_to_update.to_delete &&
                        price_product_to_update.price_product.id.is_some() {

                        diesel::delete(
                                prices_products::table
                                    .filter(prices_products::user_id.eq(param_user_id))
                                    .find(price_product_to_update.price_product.id.unwrap()))
                            .execute(connection)?;
                    } else {
                        records_to_keep.push(price_product_to_update)
                    }
                }

                let product_prices =
                    records_to_keep
                        .iter()
                        .map(|price_product| {

                            let new_price_product = NewPriceProduct {
                                user_id: Some(param_user_id),
                                product_id: Some(param_product_id),
                                ..price_product.clone().price_product
                            };

                            diesel::insert_into(prices_products::table)
                                .values(&new_price_product)
                                .on_conflict((prices_products::price_id, 
                                            prices_products::product_id))
                                .do_update()
                                .set(prices_products::amount.eq(new_price_product.amount))
                                .returning((prices_products::id, 
                                            prices_products::price_id,
                                            prices_products::product_id,
                                            prices_products::user_id,
                                            prices_products::amount))
                                .get_result::<PriceProduct>(connection)
                        })
                        .fold_results(vec![], |mut accum, value| {
                            accum.push(value);
                            accum
                        })?;

                let mut full_price_product = vec![];
                for price_product in product_prices {
                    let price = Price::find(
                            &context,
                            price_product.price_id,
                        ).map_err(|_| {
                            diesel::result::Error::NotFound
                        })?;
                    full_price_product.push(
                        FullPriceProduct {
                            price_product,
                            price
                        }
                    )
                }
                Ok(full_price_product)
            })

        }
}

impl PriceList {
    pub fn list(context: &Context) ->
        FieldResult<Self> {
            use diesel::ExpressionMethods;
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;

            let connection: &PgConnection = &context.conn;
            let param_user_id = context.user_id;

            Ok(
                PriceList{
                    data: prices
                            .filter(user_id.eq(param_user_id))
                            .load::<Price>(connection)?
                }
            )
        }
}

impl Price {
    pub fn create(context: &Context, new_price: NewPrice) ->
        FieldResult<Price> {
            use diesel::RunQueryDsl;

            let connection: &PgConnection = &context.conn;
            let param_user_id = context.user_id;

            let new_price = NewPrice {
                user_id: Some(param_user_id),
                ..new_price
            };

            Ok(diesel::insert_into(prices::table)
                .values(new_price)
                .returning((id, name, user_id))
                .get_result::<Price>(connection)?)

        }

    pub fn find(context: &Context, price_id: i32) -> 
        FieldResult<Price> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;

            let connection: &PgConnection = &context.conn;
            let param_user_id = context.user_id;

            Ok(prices
                .filter(user_id.eq(param_user_id))
                .find(price_id)
                .first(connection)?)
    }

    pub fn destroy(context: &Context, price_id: i32) 
        -> FieldResult<bool> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;

            let connection: &PgConnection = &context.conn;
            let param_user_id = context.user_id;

            diesel::delete(prices.filter(user_id.eq(param_user_id)).find(price_id))
                .execute(connection)?;
            Ok(true)
    }

    pub fn update(context: &Context, edit_price: NewPrice) 
        -> FieldResult<Price> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;

            let connection: &PgConnection = &context.conn;
            let param_user_id = context.user_id;

            let price_id = edit_price
                .id
                .ok_or(diesel::result::Error::QueryBuilderError(
                    "missing id".into(),
                ))?;

            let new_price_to_replace = NewPrice {
                user_id: Some(param_user_id),
                ..edit_price.clone()
            };

            let price =
                diesel::update(prices.filter(user_id.eq(param_user_id)).find(price_id))
                    .set(new_price_to_replace)
                    .get_result::<Price>(connection)?;

            Ok(price)
        }
}
