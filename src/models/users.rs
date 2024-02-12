use sqlx::{FromRow, SqlitePool};

#[derive(Debug, FromRow)]
pub struct User {
    user_id: i32,
    username: String,
    pass: String,
}

impl User {
    pub fn get_id(&self) -> i32 {
        self.user_id
    }

    pub fn check_pass(&self, pass: &str) -> bool {
        &self.pass == pass
    }

    pub async fn create_user(pool: &SqlitePool, user: &str, pass: &str) -> sqlx::Result<Self> {
        sqlx::query("INSERT INTO users(username, pass) VALUES (?, ?) RETURNING *")
            .bind(user)
            .bind(pass)
            .fetch_one(pool)
            .await
            .and_then(|r| User::from_row(&r))
    }

    pub async fn get_user(pool: &SqlitePool, user: &str) -> sqlx::Result<Self> {
        sqlx::query("SELECT * FROM users WHERE username = ?")
            .bind(user)
            .fetch_one(pool)
            .await
            .and_then(|r| User::from_row(&r))
    }
}
