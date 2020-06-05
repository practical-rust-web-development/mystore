use chrono::NaiveDate;
use diesel::{
    sql_types, BelongingToDsl, BoolExpressionMethods, Connection, ExpressionMethods, GroupedBy,
    PgConnection, QueryDsl, RunQueryDsl,
};
use juniper::FieldResult;

use crate::errors::MyStoreError;
use crate::models::product::{Product, PRODUCT_COLUMNS};
use crate::models::sale_product::{
    FormSaleProduct, FormSaleProducts, FullFormSaleProduct, FullSaleProduct, SaleProduct,
};
use crate::models::sale_state::Event;
use crate::models::sale_state::SaleState;
use crate::models::sale_state::SaleStateMapping;
use crate::models::Context;
use crate::schema;
use crate::schema::sale_products::dsl as sale_products_dsl;
use crate::schema::sales;
use crate::schema::sales::dsl;

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
pub struct FormSale {
    pub id: Option<i32>,
    pub sale_date: Option<NaiveDate>,
    pub user_id: Option<i32>,
    pub total: Option<f64>,
    pub bill_number: Option<String>,
    pub state: Option<SaleState>,
}

#[derive(Debug, Clone, juniper::GraphQLObject)]
pub struct FullSale {
    pub sale: Sale,
    pub sale_products: Vec<FullSaleProduct>,
}

#[derive(Debug, Clone, juniper::GraphQLObject)]
pub struct FullFormSale {
    pub sale: FormSale,
    pub sale_products: Vec<FullFormSaleProduct>,
}

#[derive(Debug, Clone, juniper::GraphQLObject)]
pub struct ListSale {
    pub data: Vec<FullSale>,
}

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
    pub fn set_state(context: &Context, sale_id: i32, event: Event) -> FieldResult<bool> {
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

    pub fn list(context: &Context, search: Option<FormSale>, limit: i32) -> FieldResult<ListSale> {
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

    pub fn show(context: &Context, sale_id: i32) -> FieldResult<FullSale> {
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

    pub fn create(
        context: &Context,
        form: FormSale,
        form_sale_products: FormSaleProducts,
    ) -> FieldResult<FullSale> {
        let conn: &PgConnection = &context.conn;

        let new_sale = FormSale {
            user_id: Some(context.user_id),
            state: Some(SaleState::Draft),
            ..form
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

            let sale_products: Result<Vec<FullSaleProduct>, _> = form_sale_products
                .data
                .into_iter()
                .map(|param_new_sale_product| {
                    let new_sale_product = FormSaleProduct {
                        sale_id: Some(sale.id),
                        ..param_new_sale_product.sale_product
                    };
                    let sale_product = diesel::insert_into(schema::sale_products::table)
                        .values(new_sale_product)
                        .returning((
                            sale_products_dsl::id,
                            sale_products_dsl::product_id,
                            sale_products_dsl::sale_id,
                            sale_products_dsl::amount,
                            sale_products_dsl::discount,
                            sale_products_dsl::tax,
                            sale_products_dsl::price,
                            sale_products_dsl::total,
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

    pub fn update(
        context: &Context,
        form: FormSale,
        form_sale_products: FormSaleProducts,
    ) -> FieldResult<FullSale> {
        let conn: &PgConnection = &context.conn;
        let sale_id = form.id.ok_or(diesel::result::Error::QueryBuilderError(
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
            .set(&form)
            .get_result::<Sale>(conn)?;

            let updated_sale_products: Result<Vec<FullSaleProduct>, _> = form_sale_products
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
                sale_products: updated_sale_products?,
            })
        })
    }

    pub fn destroy(context: &Context, sale_id: i32) -> FieldResult<bool> {
        let conn: &PgConnection = &context.conn;

        let deleted_rows = diesel::delete(
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

    fn searching_records<'a>(search: Option<FormSale>) -> BoxedQuery<'a> {
        let mut query = schema::sales::table.into_boxed::<diesel::pg::Pg>();

        if let Some(sale) = search {
            if let Some(sale_sale_date) = sale.sale_date {
                query = query.filter(dsl::sale_date.eq(sale_sale_date));
            }
            if let Some(sale_bill_number) = sale.bill_number {
                query = query.filter(dsl::bill_number.eq(sale_bill_number));
            }
        }

        query
    }
}
