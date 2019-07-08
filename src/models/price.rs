use crate::schema::prices;
use crate::schema::prices_products;
use crate::models::product::Product;

#[derive(Identifiable, Queryable, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[table_name="prices"]
pub struct Price {
    pub id: i32,
    pub name: String,
    pub user_id: i32
}

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq)]
#[table_name="prices"]
pub struct NewPrice {
    pub name: Option<String>,
    pub user_id: Option<i32>
}

#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[belongs_to(Price)]
#[belongs_to(Product)]
#[table_name="prices_products"]
pub struct PriceProduct {
    pub id: i32,
    pub price_id: i32,
    pub product_id: i32,
    pub user_id: i32,
    pub amount: Option<i32>
}

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq)]
#[table_name="prices_products"]
pub struct NewPriceProduct {
    pub id: Option<i32>,
    pub price_id: i32,
    pub product_id: i32,
    pub user_id: Option<i32>,
    pub amount: Option<i32>
}

pub struct PriceProductToUpdate {
    pub price_product: NewPriceProduct,
    pub to_delete: bool
}

use diesel::PgConnection;

impl PriceProductToUpdate {
    pub fn batch_update(records: Vec<Self>, param_user_id: i32, connection: &PgConnection) ->
        Result<Vec<PriceProduct>, diesel::result::Error> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;
            use diesel::Connection;
            use itertools::Itertools;

            connection.transaction(|| {
                let mut records_to_keep = vec![];
                for price_product_to_update in records {

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

                records_to_keep
                    .iter()
                    .map(|price_product| {

                        let new_price_product = NewPriceProduct {
                            user_id: Some(param_user_id),
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
                    })
            })

        }
}