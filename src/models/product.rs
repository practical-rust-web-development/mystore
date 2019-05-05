use crate::schema::products;

#[derive(Queryable, Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub stock: f64,
    pub price: Option<i32>
}

#[derive(Insertable)]
#[table_name="products"]
pub struct NewProduct {
    pub name: String,
    pub stock: f64,
    pub price: Option<i32>
}

impl Product {
    pub fn list() -> Vec<Product> {
        use diesel::RunQueryDsl;
        use diesel::QueryDsl;
        use crate::schema::products::dsl::*;
        use crate::db_connection::establish_connection;

        let connection = establish_connection();

        products
            .limit(10)
            .load::<Product>(&connection)
            .expect("Error loading products")
    }
}