use sqlx::{Error, MySql, Pool, mysql::MySqlPoolOptions};

pub async fn database_core(database_url: &str) -> Result<Pool<MySql>, Error> {
    let pool: Pool<MySql> = MySqlPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .connect(database_url)
        .await?;
    Ok(pool)
}
