use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(FromRow, Serialize, Deserialize)]
pub struct Category {
    pub category_id: i32,
    pub name: String,
    pub description: String,
}

impl Category {
    pub async fn get_by_id(pool: &SqlitePool, id: i32) -> sqlx::Result<Self> {
        sqlx::query("SELECT * FROM categories WHERE category_id=?")
            .bind(id)
            .fetch_one(pool)
            .await
            .and_then(|r| Category::from_row(&r))
    }

    pub async fn list(pool: &SqlitePool) -> sqlx::Result<Vec<Category>> {
        let mut res = Vec::new();
        for r in sqlx::query("SELECT * FROM categories")
            .fetch_all(pool)
            .await?
            .iter()
        {
            res.push(Category::from_row(r)?)
        }

        Ok(res)
    }

    pub async fn new(pool: &SqlitePool, name: &str, description: &str) -> sqlx::Result<Category> {
        sqlx::query("INSERT INTO categories(name, description) VALUES (?,?) RETURNING *")
            .bind(name)
            .bind(description)
            .fetch_one(pool)
            .await
            .and_then(|r| Category::from_row(&r))
    }
}
