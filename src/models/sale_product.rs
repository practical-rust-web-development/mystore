use crate::models::product::{FormProduct, Product};
use crate::models::sale::Sale;
use crate::schema::sale_products;

#[derive(Identifiable, Associations, Queryable, Debug, Clone, PartialEq)]
#[table_name = "sale_products"]
#[belongs_to(Sale)]
#[belongs_to(Product)]
#[derive(juniper::GraphQLObject)]
#[graphql(description = "Relationship between sale and products")]
pub struct SaleProduct {
    pub id: i32,
    pub product_id: i32,
    pub sale_id: i32,
    pub amount: f64,
    pub discount: i32,
    pub tax: i32,
    pub price: i32,
    pub total: f64,
}

#[derive(juniper::GraphQLObject, Debug, Clone)]
pub struct FullSaleProduct {
    pub sale_product: SaleProduct,
    pub product: Product,
}

#[derive(Insertable, Deserialize, Serialize, AsChangeset, Debug, Clone, PartialEq)]
#[table_name = "sale_products"]
#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "Relationship between sale and products")]
pub struct FormSaleProduct {
    pub id: Option<i32>,
    pub product_id: Option<i32>,
    pub sale_id: Option<i32>,
    pub amount: Option<f64>,
    pub discount: Option<i32>,
    pub tax: Option<i32>,
    pub price: Option<i32>,
    pub total: Option<f64>,
}

#[derive(juniper::GraphQLInputObject, Debug, Clone)]
pub struct FullFormSaleProduct {
    pub sale_product: FormSaleProduct,
    pub product: FormProduct,
}

#[derive(juniper::GraphQLInputObject)]
pub struct FormSaleProducts {
    pub data: Vec<FullFormSaleProduct>,
}
