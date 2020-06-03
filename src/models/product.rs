use crate::models::price::{Price, PriceProduct, FullPriceProduct, FormPriceProductsToUpdate};
use crate::schema::products;
use diesel::BelongingToDsl;
use diesel::PgConnection;
use juniper::FieldResult;
use crate::models::Context;

#[derive(Debug, Clone, juniper::GraphQLObject)]
pub struct ListProduct {
    pub data: Vec<FullProduct>
}

#[derive(Debug, Clone, Serialize, Deserialize, juniper::GraphQLObject)]
pub struct FullProduct {
    pub product: Product,
    pub price_products: Vec<FullPriceProduct>
}

#[derive(
    Identifiable, Queryable, Serialize, Deserialize, Debug, Clone, PartialEq
)]
#[table_name = "products"]
#[derive(juniper::GraphQLObject)]
#[graphql(description = "Product")]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub stock: f64,
    pub cost: Option<i32>,
    pub description: Option<String>,
    pub user_id: i32,
}

pub type ProductColumns = (
    products::id,
    products::name,
    products::stock,
    products::cost,
    products::description,
    products::user_id,
);

pub const PRODUCT_COLUMNS: ProductColumns = (
    products::id,
    products::name,
    products::stock,
    products::cost,
    products::description,
    products::user_id,
);

#[derive(
    Insertable,
    Deserialize,
    Serialize,
    AsChangeset,
    Debug,
    Clone,
    PartialEq,
    juniper::GraphQLInputObject,
)]
#[table_name = "products"]
pub struct FormProduct {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub stock: Option<f64>,
    pub cost: Option<i32>,
    pub description: Option<String>,
    pub user_id: Option<i32>,
}

use crate::models::price::PriceProductToUpdate;

impl Product {

    pub fn list(context: &Context, search: String, limit: i32, rank: f64) -> FieldResult<ListProduct> {
        use crate::schema;
        use crate::schema::products::dsl::*;
        use diesel::pg::Pg;
        use diesel::BoolExpressionMethods;
        use diesel::ExpressionMethods;
        use diesel::GroupedBy;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use diesel_full_text_search::{plainto_tsquery, TsRumExtensions, TsVectorExtensions};

        let connection: &PgConnection = &context.conn;
        let mut query = schema::products::table.into_boxed::<Pg>();

        if !search.is_empty() {
            query = query
                .filter(text_searchable_product_col.matches(plainto_tsquery(search.clone())))
                .order((
                    product_rank.desc(),
                    text_searchable_product_col.distance(plainto_tsquery(search)),
                ));
        } else {
            query = query.order(product_rank.desc());
        }
        let query_products = query
            .select(PRODUCT_COLUMNS)
            .filter(user_id.eq(context.user_id).and(product_rank.le(rank)))
            .limit(i64::from(limit))
            .load::<Product>(connection)?;

        let products_with_prices = PriceProduct::belonging_to(&query_products)
            .inner_join(schema::prices::table)
            .load::<(PriceProduct, Price)>(connection)?
            .grouped_by(&query_products);

        let vec_full_product = query_products
            .into_iter()
            .zip(products_with_prices)
            .map(|tuple_product| {
                let full_price_product = tuple_product
                    .1
                    .iter()
                    .map(|tuple_price_product| FullPriceProduct {
                        price_product: tuple_price_product.0.clone(),
                        price: tuple_price_product.1.clone(),
                    })
                    .collect();
                FullProduct {
                    product: tuple_product.0.clone(),
                    price_products: full_price_product,
                }
            })
            .collect();
        
        Ok(ListProduct {
            data: vec_full_product,
        })
    }

    pub fn create(
        context: &Context,
        form: FormProduct,
        prices: FormPriceProductsToUpdate,
    ) -> FieldResult<FullProduct> {
        use diesel::RunQueryDsl;

        let connection: &PgConnection = &context.conn;
        let user_id = context.user_id;

        let new_product = FormProduct {
            user_id: Some(user_id),
            ..form
        };

        let product = diesel::insert_into(products::table)
            .values(new_product)
            .returning(PRODUCT_COLUMNS)
            .get_result::<Product>(connection)?;

        let price_products =
            PriceProductToUpdate::batch_update(&context, prices, product.id)?;

        Ok(FullProduct{product, price_products})
    }

    pub fn show(context: &Context, product_id: i32) -> FieldResult<FullProduct> {
        use crate::schema;
        use crate::schema::products::dsl::*;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let connection: &PgConnection = &context.conn;
        let product: Product = schema::products::table
            .select(PRODUCT_COLUMNS)
            .filter(user_id.eq(context.user_id))
            .find(product_id)
            .first(connection)?;

        let products_with_prices =
            PriceProduct::belonging_to(&product)
            .inner_join(schema::prices::table)
            .load::<(PriceProduct, Price)>(connection)?
            .iter()
            .map(|tuple_price_product| 
                FullPriceProduct {
                    price_product: tuple_price_product.0.clone(),
                    price: tuple_price_product.1.clone()
                }
            )
            .collect();

        Ok(FullProduct {
            product,
            price_products: products_with_prices
        })
    }

    pub fn destroy(
        context: &Context,
        id: i32
    ) -> FieldResult<bool> {
        use crate::schema::products::dsl;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let connection: &PgConnection = &context.conn;
        let param_user_id = context.user_id;
        diesel::delete(
            dsl::products
                .filter(dsl::user_id.eq(param_user_id))
                .find(id),
        )
        .execute(connection)?;
        Ok(true)
    }

    pub fn update(
        context: &Context,
        form: FormProduct,
        prices: FormPriceProductsToUpdate,
    ) -> FieldResult<FullProduct> {
        use crate::schema::products::dsl;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        let connection: &PgConnection = &context.conn;
        let param_user_id = context.user_id;
        let product_id = form
            .id
            .ok_or(diesel::result::Error::QueryBuilderError(
                "missing id".into(),
            ))?;

        let new_product_to_replace = FormProduct {
            user_id: Some(param_user_id),
            ..form.clone()
        };

        let product =
            diesel::update(
                dsl::products
                    .filter(dsl::user_id.eq(param_user_id))
                    .find(product_id),
            )
            .set(&new_product_to_replace)
            .returning(PRODUCT_COLUMNS)
            .get_result::<Product>(connection)?;

        let price_products =
            PriceProductToUpdate::batch_update(&context, prices, product_id)?;

        Ok(FullProduct{product, price_products})
    }
}

impl PartialEq<Product> for FormProduct {
    fn eq(&self, other: &Product) -> bool {
        let new_product = self.clone();
        let product = other.clone();
        new_product.name == Some(product.name)
            && new_product.stock == Some(product.stock)
            && new_product.cost == product.cost
            && new_product.description == product.description
    }
}
