pub mod product;
pub mod user;

#[derive(Serialize, Deserialize)]
pub struct MyStoreResponder<T>(pub T);