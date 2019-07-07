use crate::schema::products;
use diesel::PgConnection;

#[derive(Serialize, Deserialize)]
pub struct ProductList(pub Vec<Product>);

#[derive(Queryable, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub stock: f64,
    pub price: Option<i32>,
    pub description: Option<String>,
    pub user_id: i32
}

type ProductColumns = (
    products::id,
    products::name,
    products::stock,
    products::price,
    products::description,
    products::user_id
);

const PRODUCT_COLUMNS: ProductColumns = (
    products::id,
    products::name,
    products::stock,
    products::price,
    products::description,
    products::user_id
);

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq)]
#[table_name="products"]
pub struct NewProduct {
    pub name: Option<String>,
    pub stock: Option<f64>,
    pub price: Option<i32>,
    pub description: Option<String>,
    pub user_id: Option<i32>
}

impl ProductList {
    pub fn list(param_user_id: i32, search: &str, rank: f64, connection: &PgConnection) -> Self {
        use diesel::RunQueryDsl;
        use diesel::ExpressionMethods;
        use diesel::QueryDsl;
        use diesel::pg::Pg;
        use diesel::BoolExpressionMethods;
        use crate::schema::products::dsl::*;
        use crate::schema;
        use diesel_full_text_search::{plainto_tsquery, TsRumExtensions, TsVectorExtensions};

        let mut query = schema::products::table.into_boxed::<Pg>();

        if !search.is_empty() {
            query = query
                .filter(text_searchable_product_col.matches(plainto_tsquery(search)))
                .order((product_rank.desc(), 
                        text_searchable_product_col.distance(plainto_tsquery(search))));
        } else {
            query = query.order(product_rank.desc());
        }
        let result = query
            .select(PRODUCT_COLUMNS)
            .filter(user_id.eq(param_user_id).and(product_rank.le(rank)))
            .limit(10)
            .load::<Product>(connection)
            .expect("Error loading products");

        ProductList(result)
    }
}

impl NewProduct {
    pub fn create(&self, param_user_id: i32, connection: &PgConnection) -> Result<Product, diesel::result::Error> {
        use diesel::RunQueryDsl;

        let new_product = NewProduct {
            user_id: Some(param_user_id),
            ..self.clone()
        };

        diesel::insert_into(products::table)
            .values(new_product)
            .returning(PRODUCT_COLUMNS)
            .get_result::<Product>(connection)
    }
}

impl Product {
    pub fn find(product_id: &i32, param_user_id: i32, connection: &PgConnection) -> Result<Product, diesel::result::Error> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use diesel::ExpressionMethods;
        use crate::schema;
        use crate::schema::products::dsl::*;

        schema::products::table
            .select(PRODUCT_COLUMNS)
            .filter(user_id.eq(param_user_id))
            .find(product_id)
            .first(connection)
    }

    pub fn destroy(id: &i32, param_user_id: i32, connection: &PgConnection) -> Result<(), diesel::result::Error> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use diesel::ExpressionMethods;
        use crate::schema::products::dsl;

        diesel::delete(dsl::products.filter(dsl::user_id.eq(param_user_id)).find(id))
            .execute(connection)?;
        Ok(())
    }

    pub fn update(id: &i32, param_user_id: i32, new_product: &NewProduct, connection: &PgConnection) ->
     Result<(), diesel::result::Error> {
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
        Ok(())
    }
}

impl PartialEq<Product> for NewProduct {
    fn eq(&self, other: &Product) -> bool {
        let new_product = self.clone();
        let product = other.clone();
        new_product.name == Some(product.name) &&
        new_product.stock == Some(product.stock) &&
        new_product.price == product.price &&
        new_product.description == product.description
    }
}