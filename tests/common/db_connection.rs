use diesel::pg::PgConnection;
use diesel::r2d2::{ Pool, ConnectionManager, PoolError };

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

fn init_pool(database_url: &str) -> Result<PgPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub fn establish_connection() -> PgPool {
    init_pool(dotenv!("DATABASE_URL_TEST")).expect("Failed to create pool")
}