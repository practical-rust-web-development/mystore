use crate::models::price::Price;
use crate::models::price::PriceProduct;
use crate::schema::products;
use diesel::PgConnection;
use diesel::BelongingToDsl;

#[derive(Serialize, Deserialize)]
pub struct ProductList(pub Vec<(Product, Vec<(PriceProduct, Price)>)>);

#[derive(Identifiable, Queryable, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[derive(juniper::GraphQLObject)]
#[table_name="products"]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub stock: f64,
    pub cost: Option<i32>,
    pub description: Option<String>,
    pub user_id: i32
}

pub type ProductColumns = (
    products::id,
    products::name,
    products::stock,
    products::cost,
    products::description,
    products::user_id
);

pub const PRODUCT_COLUMNS: ProductColumns = (
    products::id,
    products::name,
    products::stock,
    products::cost,
    products::description,
    products::user_id
);

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq)]
#[derive(juniper::GraphQLInputObject)]
#[table_name="products"]
pub struct NewProduct {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub stock: Option<f64>,
    pub cost: Option<i32>,
    pub description: Option<String>,
    pub user_id: Option<i32>
}

use crate::errors::MyStoreError;

impl ProductList {
    pub fn list(param_user_id: i32, search: &str, rank: f64, connection: &PgConnection) ->
        Result<Self, MyStoreError> {
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;
            use diesel::QueryDsl;
            use diesel::pg::Pg;
            use diesel::BoolExpressionMethods;
            use diesel::GroupedBy;
            use diesel_full_text_search::{plainto_tsquery, TsRumExtensions, TsVectorExtensions};
            use crate::schema::products::dsl::*;
            use crate::schema;

            let mut query = schema::products::table.into_boxed::<Pg>();

            if !search.is_empty() {
                query = query
                    .filter(text_searchable_product_col.matches(plainto_tsquery(search)))
                    .order((product_rank.desc(), 
                            text_searchable_product_col.distance(plainto_tsquery(search))));
            } else {
                query = query.order(product_rank.desc());
            }
            let query_products = query
                .select(PRODUCT_COLUMNS)
                .filter(user_id.eq(param_user_id).and(product_rank.le(rank)))
                .limit(10)
                .load::<Product>(connection)?;

            let products_with_prices =
                PriceProduct::belonging_to(&query_products)
                    .inner_join(schema::prices::table)
                    .load::<(PriceProduct, Price)>(connection)?
                    .grouped_by(&query_products);

            Ok(
                ProductList(
                    query_products
                        .into_iter()
                        .zip(products_with_prices)
                        .collect::<Vec<_>>()
                )
            )
    }
}

use crate::models::price::PriceProductToUpdate;

impl NewProduct {
    pub fn create(&self, param_user_id: i32, prices: Vec<PriceProductToUpdate>, connection: &PgConnection) ->
        Result<(Product, Vec<PriceProduct>), MyStoreError> {
            use diesel::RunQueryDsl;

            let new_product = NewProduct {
                user_id: Some(param_user_id),
                ..self.clone()
            };

            let product = 
                diesel::insert_into(products::table)
                    .values(new_product)
                    .returning(PRODUCT_COLUMNS)
                    .get_result::<Product>(connection)?;

            let price_products = 
                PriceProductToUpdate::batch_update(
                    prices,
                    product.id,
                    param_user_id,
                    connection)?;

            Ok((product, price_products))
        }
}

impl Product {
    pub fn find(product_id: &i32, param_user_id: i32, connection: &PgConnection)
        -> Result<(Product, Vec<PriceProduct>), MyStoreError> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;
            use crate::schema;
            use crate::schema::products::dsl::*;

            let product: Product =
                schema::products::table
                    .select(PRODUCT_COLUMNS)
                    .filter(user_id.eq(param_user_id))
                    .find(product_id)
                    .first(connection)?;
            
            let products_with_prices =
                PriceProduct::belonging_to(&product)
                    .load::<PriceProduct>(connection)?;

            Ok((product, products_with_prices))
    }

    pub fn destroy(id: &i32, param_user_id: i32, connection: &PgConnection) 
        -> Result<(), MyStoreError> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;
            use crate::schema::products::dsl;

            diesel::delete(dsl::products.filter(dsl::user_id.eq(param_user_id)).find(id))
                .execute(connection)?;
            Ok(())
    }

    pub fn update(id: i32, param_user_id: i32, new_product: NewProduct, prices: Vec<PriceProductToUpdate>, connection: &PgConnection) 
        -> Result<(), MyStoreError> {
            use diesel::QueryDsl;
            use diesel::RunQueryDsl;
            use diesel::ExpressionMethods;
            use crate::schema::products::dsl;

            let new_product_to_replace = NewProduct {
                user_id: Some(param_user_id),
                ..new_product.clone()
            };

            diesel::update(dsl::products.filter(dsl::user_id.eq(param_user_id)).find(id))
                .set(new_product_to_replace)
                .execute(connection)?;

            PriceProductToUpdate::batch_update(
                prices,
                id,
                param_user_id,
                connection)?;

            Ok(())
    }
}

impl PartialEq<Product> for NewProduct {
    fn eq(&self, other: &Product) -> bool {
        let new_product = self.clone();
        let product = other.clone();
        new_product.name == Some(product.name) &&
        new_product.stock == Some(product.stock) &&
        new_product.cost == product.cost &&
        new_product.description == product.description
    }
}