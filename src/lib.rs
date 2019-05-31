#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate serde;
extern crate serde_json;
#[macro_use] 
extern crate serde_derive;

extern crate actix;
extern crate actix_web;
extern crate bcrypt;
extern crate jsonwebtoken as jwt;
extern crate csrf_token;

#[macro_use]
extern crate dotenv_codegen;

#[macro_use] extern crate log;
extern crate env_logger;

extern crate actix_http;

pub mod schema;
pub mod db_connection;
pub mod models;
pub mod handlers;
pub mod errors;
pub mod utils;