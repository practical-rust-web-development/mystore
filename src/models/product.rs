use crate::schema::products;
use diesel::PgConnection;

#[derive(Serialize, Deserialize)]
pub struct ProductList(pub Vec<Product>);

#[derive(Queryable, Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub stock: f64,
    pub price: Option<i32>
}

#[derive(Insertable, Deserialize, AsChangeset)]
#[table_name="products"]
pub struct NewProduct {
    pub name: Option<String>,
    pub stock: Option<f64>,
    pub price: Option<i32>
}

impl ProductList {
    pub fn list(connection: &PgConnection) -> Self {
        use diesel::RunQueryDsl;
        use diesel::QueryDsl;
        use crate::schema::products::dsl::*;

        let result = 
            products
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
            .get_result(connection)
    }
}

impl Product {
    pub fn find(id: &i32, connection: &PgConnection) -> Result<Product, diesel::result::Error> {
        use diesel::QueryDsl;
        use diesel::RunQueryDsl;

        products::table.find(id).first(connection)
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