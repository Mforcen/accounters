use log::{error, info};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

pub mod models;

pub async fn create_db(db_url: &str) -> sqlx::Result<SqlitePool> {
    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        println!("Creating database {}", db_url);
        match Sqlite::create_database(db_url).await {
            Ok(_) => info!("create db success"),
            Err(e) => {
                error!("Cannot create db at {db_url}: {e:?}");
                panic!("Cannot create db at {db_url}: {e:?}")
            }
        };
    } else {
        println!("Database already exists")
    }

    let db = SqlitePool::connect(db_url).await?;
    sqlx::migrate!().run(&db).await?;

    Ok(db)
}
