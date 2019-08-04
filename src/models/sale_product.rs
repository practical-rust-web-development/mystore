use crate::schema::sale_products;
use crate::models::sale::Sale;
use crate::models::product::Product;

#[derive(Identifiable, Associations, Queryable, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[table_name="sale_products"]
#[belongs_to(Sale)]
#[belongs_to(Product)]
#[derive(juniper::GraphQLObject)]
#[graphql(description="Relationship between sale and products")]
pub struct SaleProduct {
    pub id: i32,
    pub product_id: i32,
    pub sale_id: i32,
    pub amount: f64,
    pub discount: i32,
    pub tax: i32,
    pub price: i32,
    pub total: f64
}

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq)]
#[table_name="sale_products"]
#[derive(juniper::GraphQLInputObject)]
#[graphql(description="Relationship between sale and products")]
pub struct NewSaleProduct {
    pub product_id: i32,
    pub sale_id: Option<i32>,
    pub amount: f64,
    pub discount: i32,
    pub tax: i32,
    pub price: i32,
    pub total: f64
}

#[derive(juniper::GraphQLInputObject)]
pub struct NewSaleProducts{ pub data: Vec<NewSaleProduct> }