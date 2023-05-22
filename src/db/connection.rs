use std::env;

use diesel::r2d2::{ConnectionManager, Pool, PoolError};
use diesel::sqlite::SqliteConnection;
use r2d2::PooledConnection;

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
pub type SqlitePooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub fn establish_connection() -> SqlitePool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    init_pool(&database_url).expect("Failed to create pool.")
}

fn init_pool(database_url: &str) -> Result<SqlitePool, PoolError> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder().build(manager)
}

pub fn get_connection(pool: &SqlitePool) -> Result<SqlitePooledConnection, PoolError> {
    let _pool = pool.get().unwrap();
    Ok(_pool)
}
