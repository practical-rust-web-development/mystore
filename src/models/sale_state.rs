#[derive(DbEnum, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(juniper::GraphQLEnum)]
pub enum SaleState {
    Draft,
    Approved,
    NotPayed,
    Payed,
    Cancelled
}