use crate::errors::MyStoreError;
use crate::models::product::{Product, PRODUCT_COLUMNS};
use crate::models::sale_state::SaleState;
use crate::schema;
use crate::schema::sale_products;
use crate::schema::sales;
use chrono::NaiveDate;
use diesel::sql_types;
use diesel::BelongingToDsl;
use diesel::PgConnection;
use juniper::FieldResult;
use crate::models::Context;
use crate::models::sale_state::Event;

#[derive(Identifiable, Queryable, Debug, Clone, PartialEq)]
#[table_name = "sales"]
#[derive(juniper::GraphQLObject)]
#[graphql(description = "Sale Bill")]
pub struct Sale {
    pub id: i32,
    pub user_id: i32,
    pub sale_date: NaiveDate,
    pub total: f64,
    pub bill_number: Option<String>,
    pub state: SaleState,
}

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq)]
#[table_name = "sales"]
#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Sale Bill")]
pub struct NewSale {
    pub id: Option<i32>,
    pub sale_date: Option<NaiveDate>,
    pub user_id: Option<i32>,
    pub total: Option<f64>,
    pub bill_number: Option<String>,
    pub state: Option<SaleState>,
}

use crate::models::sale_product::{
    FullNewSaleProduct, FullSaleProduct, NewSaleProduct, NewSaleProducts, SaleProduct,
};

#[derive(Debug, Clone, juniper::GraphQLObject)]
pub struct FullSale {
    pub sale: Sale,
    pub sale_products: Vec<FullSaleProduct>,
}

#[derive(Debug, Clone, juniper::GraphQLObject)]
pub struct FullNewSale {
    pub sale: NewSale,
    pub sale_products: Vec<FullNewSaleProduct>,
}

#[derive(Debug, Clone, juniper::GraphQLObject)]
pub struct ListSale {
    pub data: Vec<FullSale>,
}

use crate::models::sale_state::SaleStateMapping;

type BoxedQuery<'a> = diesel::query_builder::BoxedSelectStatement<
    'a,
    (
        sql_types::Integer,
        sql_types::Integer,
        sql_types::Date,
        sql_types::Float8,
        sql_types::Nullable<sql_types::Text>,
        SaleStateMapping,
    ),
    schema::sales::table,
    diesel::pg::Pg,
>;

impl Sale {
    fn searching_records<'a>(search: Option<NewSale>) -> BoxedQuery<'a> {
        use crate::schema::sales::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;

        let mut query = schema::sales::table.into_boxed::<diesel::pg::Pg>();

        if let Some(sale) = search {
            if let Some(sale_sale_date) = sale.sale_date {
                query = query.filter(sale_date.eq(sale_sale_date));
            }
            if let Some(sale_bill_number) = sale.bill_number {
                query = query.filter(bill_number.eq(sale_bill_number));
            }
        }

        query
    }

    pub fn set_state(context: &Context, sale_id: i32, event: Event) -> FieldResult<bool> {
        use crate::schema::sales::dsl;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let conn: &PgConnection = &context.conn;
        let sale_query_builder = dsl::sales
            .filter(dsl::user_id.eq(context.user_id))
            .find(sale_id);

        let sale = sale_query_builder.first::<Sale>(conn)?;
        let sale_state = sale.state.next(event)?;

        diesel::update(sale_query_builder)
            .set(dsl::state.eq(sale_state))
            .get_result::<Sale>(conn)?;

        Ok(true)
    }

    pub fn list_sale(context: &Context, search: Option<NewSale>, limit: i32) -> FieldResult<ListSale> {
        use diesel::{ExpressionMethods, GroupedBy, QueryDsl, RunQueryDsl};
        let conn: &PgConnection = &context.conn;
        let query = Sale::searching_records(search);

        let query_sales: Vec<Sale> = query
            .filter(sales::dsl::user_id.eq(context.user_id))
            .limit(limit.into())
            .load::<Sale>(conn)?;

        let query_sale_products = SaleProduct::belonging_to(&query_sales)
            .inner_join(schema::products::table)
            .select((
                (
                    schema::sale_products::id,
                    schema::sale_products::product_id,
                    schema::sale_products::sale_id,
                    schema::sale_products::amount,
                    schema::sale_products::discount,
                    schema::sale_products::tax,
                    schema::sale_products::price,
                    schema::sale_products::total,
                ),
                PRODUCT_COLUMNS,
            ))
            .load::<(SaleProduct, Product)>(conn)?
            .grouped_by(&query_sales);

        let tuple_full_sale: Vec<(Sale, Vec<(SaleProduct, Product)>)> = query_sales
            .into_iter()
            .zip(query_sale_products)
            .collect::<Vec<(Sale, Vec<(SaleProduct, Product)>)>>();

        let vec_full_sale = tuple_full_sale
            .iter()
            .map(|tuple_sale| {
                let full_sale_product = tuple_sale
                    .1
                    .iter()
                    .map(|tuple_sale_product| FullSaleProduct {
                        sale_product: tuple_sale_product.0.clone(),
                        product: tuple_sale_product.1.clone(),
                    })
                    .collect();
                FullSale {
                    sale: tuple_sale.0.clone(),
                    sale_products: full_sale_product,
                }
            })
            .collect();

        Ok(ListSale {
            data: vec_full_sale,
        })
    }

    pub fn sale(context: &Context, sale_id: i32) -> FieldResult<FullSale> {
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

        let conn: &PgConnection = &context.conn;
        let sale: Sale = schema::sales::table
            .filter(sales::dsl::user_id.eq(context.user_id))
            .find(sale_id)
            .first::<Sale>(conn)?;

        let sale_products = SaleProduct::belonging_to(&sale)
            .inner_join(schema::products::table)
            .select((
                (
                    schema::sale_products::id,
                    schema::sale_products::product_id,
                    schema::sale_products::sale_id,
                    schema::sale_products::amount,
                    schema::sale_products::discount,
                    schema::sale_products::tax,
                    schema::sale_products::price,
                    schema::sale_products::total,
                ),
                PRODUCT_COLUMNS,
            ))
            .load::<(SaleProduct, Product)>(conn)?
            .iter()
            .map(|tuple| FullSaleProduct {
                sale_product: tuple.0.clone(),
                product: tuple.1.clone(),
            })
            .collect();
        Ok(FullSale {
            sale,
            sale_products,
        })
    }

    pub fn create_sale(
        context: &Context,
        param_new_sale: NewSale,
        param_new_sale_products: NewSaleProducts,
    ) -> FieldResult<FullSale> {
        use diesel::{Connection, QueryDsl, RunQueryDsl};

        let conn: &PgConnection = &context.conn;

        let new_sale = NewSale {
            user_id: Some(context.user_id),
            state: Some(SaleState::Draft),
            ..param_new_sale
        };

        conn.transaction(|| {
            let sale = diesel::insert_into(schema::sales::table)
                .values(new_sale)
                .returning((
                    sales::dsl::id,
                    sales::dsl::user_id,
                    sales::dsl::sale_date,
                    sales::dsl::total,
                    sales::dsl::bill_number,
                    sales::dsl::state,
                ))
                .get_result::<Sale>(conn)?;

            let sale_products: Result<Vec<FullSaleProduct>, _> = param_new_sale_products
                .data
                .into_iter()
                .map(|param_new_sale_product| {
                    let new_sale_product = NewSaleProduct {
                        sale_id: Some(sale.id),
                        ..param_new_sale_product.sale_product
                    };
                    let sale_product = diesel::insert_into(schema::sale_products::table)
                        .values(new_sale_product)
                        .returning((
                            sale_products::dsl::id,
                            sale_products::dsl::product_id,
                            sale_products::dsl::sale_id,
                            sale_products::dsl::amount,
                            sale_products::dsl::discount,
                            sale_products::dsl::tax,
                            sale_products::dsl::price,
                            sale_products::dsl::total,
                        ))
                        .get_result::<SaleProduct>(conn);

                    if let Some(param_product_id) = param_new_sale_product.sale_product.product_id {
                        let product = schema::products::table
                            .select(PRODUCT_COLUMNS)
                            .find(param_product_id)
                            .first(conn);

                        Ok(FullSaleProduct {
                            sale_product: sale_product?,
                            product: product?,
                        })
                    } else {
                        Err(MyStoreError::PGConnectionError)
                    }
                })
                .collect();

            Ok(FullSale {
                sale,
                sale_products: sale_products?,
            })
        })
    }

    pub fn update_sale(
        context: &Context,
        param_sale: NewSale,
        param_sale_products: NewSaleProducts,
    ) -> FieldResult<FullSale> {
        use crate::schema::sales::dsl;
        use diesel::BoolExpressionMethods;
        use diesel::Connection;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let conn: &PgConnection = &context.conn;
        let sale_id = param_sale
            .id
            .ok_or(diesel::result::Error::QueryBuilderError(
                "missing id".into(),
            ))?;

        conn.transaction(|| {
            let sale = diesel::update(
                dsl::sales
                    .filter(
                        dsl::user_id
                            .eq(context.user_id)
                            .and(dsl::state.eq(SaleState::Draft)),
                    )
                    .find(sale_id),
            )
            .set(&param_sale)
            .get_result::<Sale>(conn)?;

            let sale_products: Result<Vec<FullSaleProduct>, _> = param_sale_products
                .data
                .into_iter()
                .map(|param_sale_product| {
                    let sale_product = diesel::update(schema::sale_products::table)
                        .set(&param_sale_product.sale_product)
                        .get_result::<SaleProduct>(conn);

                    if let Some(param_product_id) = param_sale_product.sale_product.product_id {
                        let product = schema::products::table
                            .select(PRODUCT_COLUMNS)
                            .find(param_product_id)
                            .first(conn);

                        Ok(FullSaleProduct {
                            sale_product: sale_product?,
                            product: product?,
                        })
                    } else {
                        Err(MyStoreError::PGConnectionError)
                    }
                })
                .collect();

            Ok(FullSale {
                sale,
                sale_products: sale_products?,
            })
        })
    }

    pub fn destroy_sale(context: &Context, sale_id: i32) -> FieldResult<bool> {
        use crate::schema::sales::dsl;
        use diesel::BoolExpressionMethods;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let conn: &PgConnection = &context.conn;
        let deleted_rows = 
            diesel::delete(
                dsl::sales
                    .filter(
                        dsl::user_id
                            .eq(context.user_id)
                            .and(dsl::state.eq(SaleState::Draft)),
                    )
                    .find(sale_id),
            )
            .execute(conn)?;
        Ok(deleted_rows == 1)
    }
}