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
    pub description: Option<String>
}

type ProductColumns = (
    products::id,
    products::name,
    products::stock,
    products::price,
    products::description
);

const PRODUCT_COLUMNS: ProductColumns = (
    products::id,
    products::name,
    products::stock,
    products::price,
    products::description
);

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq)]
#[table_name="products"]
pub struct NewProduct {
    pub name: Option<String>,
    pub stock: Option<f64>,
    pub price: Option<i32>,
    pub description: Option<String>
}

impl ProductList {
    pub fn list(connection: &PgConnection, search: &str) -> Self {
        use diesel::RunQueryDsl;
        use diesel::QueryDsl;
        use diesel::pg::Pg;
        use crate::schema::products::dsl::*;
        use crate::schema;
        use diesel_full_text_search::{plainto_tsquery, TsVectorExtensions};

        let mut query = schema::products::table.into_boxed::<Pg>();

        if !search.is_empty() {
            query = query
                .filter(text_searchable_product_col.matches(plainto_tsquery(search)));
        } 
        let result = query
            .select(PRODUCT_COLUMNS)
            .limit(10)
            .load::<Product>(connection)
            .expect("Error loading products");

        ProductList(result)
    }
}

impl NewProduct {
    pub fn create(&self, connection: &PgConnection) -> Result<Product, diesel::result::Error> {
        use diesel::RunQueryDsl;

        diesel::insert_into(products::table)
            .values(self)
            .on_conflict_do_nothing()
            .returning(PRODUCT_COLUMNS)
            .get_result::<Product>(connection)
    }
}

impl Product {
    pub fn find(id: &i32, connection: &PgConnection) -> Result<Product, diesel::result::Error> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        products::table.find(id).select(PRODUCT_COLUMNS).first(connection)
    }

    pub fn destroy(id: &i32, connection: &PgConnection) -> Result<(), diesel::result::Error> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use crate::schema::products::dsl;

        diesel::delete(dsl::products.find(id)).execute(connection)?;
        Ok(())
    }

    pub fn update(id: &i32, new_product: &NewProduct, connection: &PgConnection) ->
     Result<(), diesel::result::Error> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;
        use crate::schema::products::dsl;

        diesel::update(dsl::products.find(id))
            .set(new_product)
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