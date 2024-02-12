use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

pub mod models;

pub async fn create_db(db_url: &str) -> sqlx::Result<SqlitePool> {
    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        println!("Creating database {}", db_url);
        match Sqlite::create_database(db_url).await {
            Ok(_) => println!("create db success"),
            Err(e) => panic!("Cannot create db: {e:?}"),
        };
    } else {
        println!("Database already exists")
    }

    let db = SqlitePool::connect(db_url).await?;
    sqlx::migrate!().run(&db).await?;

    Ok(db)
}
