use crate::schema::prices;
use crate::schema::prices::dsl::*;
use crate::schema::prices_products;
use crate::models::product::Product;

#[derive(Serialize, Deserialize)]
pub struct PriceList(pub Vec<Price>);

#[derive(Identifiable, Queryable, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[table_name="prices"]
#[derive(juniper::GraphQLObject)]
pub struct Price {
    pub id: i32,
    pub name: String,
    pub user_id: i32
}

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq)]
#[table_name="prices"]
#[derive(juniper::GraphQLInputObject)]
pub struct NewPrice {
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
    pub fn batch_update(records: NewPriceProductsToUpdate, param_product_id: i32, param_user_id: i32, connection: &PgConnection) ->
        Result<Vec<FullPriceProduct>, diesel::result::Error> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;
            use diesel::Connection;
            use itertools::Itertools;

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
                        price_product.price_id, 
                        param_user_id, 
                        connection).map_err(|_| {
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

use crate::errors::MyStoreError;

impl PriceList {
    pub fn list(param_user_id: i32, connection: &PgConnection) ->
        Result<Self, MyStoreError> {
            use diesel::ExpressionMethods;
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;

            Ok(PriceList(prices
                .filter(user_id.eq(param_user_id))
                .load::<Price>(connection)?))
        }
}

impl NewPrice {
    pub fn create(&self, param_user_id: i32, connection: &PgConnection) ->
        Result<Price, MyStoreError> {
            use diesel::RunQueryDsl;

            let new_price = NewPrice {
                user_id: Some(param_user_id),
                ..self.clone()
            };

            Ok(diesel::insert_into(prices::table)
                .values(new_price)
                .returning((id, name, user_id))
                .get_result::<Price>(connection)?)

        }
}

impl Price {
    pub fn find(price_id: i32, param_user_id: i32, connection: &PgConnection) -> 
        Result<Price, MyStoreError> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;

            Ok(prices
                .filter(user_id.eq(param_user_id))
                .find(price_id)
                .first(connection)?)
    }

    pub fn destroy(price_id: i32, param_user_id: i32, connection: &PgConnection) 
        -> Result<(), MyStoreError> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;

            diesel::delete(prices.filter(user_id.eq(param_user_id)).find(price_id))
                .execute(connection)?;
            Ok(())
    }

    pub fn update(price_id: i32, param_user_id: i32, new_price: NewPrice, connection: &PgConnection) 
        -> Result<(), MyStoreError> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;

            let new_price_to_replace = NewPrice {
                user_id: Some(param_user_id),
                ..new_price.clone()
            };

            diesel::update(prices.filter(user_id.eq(param_user_id)).find(price_id))
                .set(new_price_to_replace)
                .execute(connection)?;

            Ok(())
        }
}
