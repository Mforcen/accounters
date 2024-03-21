use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Result, SqlitePool};

use super::{rules::Rule, transaction::Transaction};

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct Account {
    account_id: i32,
    user: i32,
    account_name: String,
}

impl Account {
    pub fn get_id(&self) -> i32 {
        self.account_id
    }

    pub fn get_user(&self) -> i32 {
        self.user
    }

    pub fn get_account_name(&self) -> &str {
        self.account_name.as_str()
    }

    pub async fn set_account_name(&mut self, pool: &SqlitePool, name: &str) -> Result<()> {
        sqlx::query("UPDATE accounts SET account_name=? WHERE account_id=?")
            .bind(name)
            .bind(self.account_id)
            .execute(pool)
            .await?;
        self.account_name = name.to_string();
        Ok(())
    }

    pub async fn get_by_id(pool: &SqlitePool, id: i32) -> Result<Self> {
        sqlx::query("SELECT * FROM accounts WHERE account_id=?")
            .bind(id)
            .fetch_one(pool)
            .await
            .and_then(|r| Account::from_row(&r))
    }

    pub async fn new(pool: &SqlitePool, user: i32, name: &str) -> Result<Self> {
        let row = sqlx::query("INSERT INTO accounts(user, account_name) VALUES (?,?) RETURNING *")
            .bind(user)
            .bind(name)
            .fetch_one(pool)
            .await?;
        Self::from_row(&row)
    }

    pub async fn list(pool: &SqlitePool, user: i32) -> Result<Vec<Self>> {
        let rows = sqlx::query("SELECT * FROM accounts WHERE user=?")
            .bind(user)
            .fetch_all(pool)
            .await?;
        let mut res = Vec::new();
        for r in &rows {
            res.push(Account::from_row(r)?)
        }
        Ok(res)
    }

    pub async fn recategorize_transactions(
        &self,
        pool: &SqlitePool,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let rules = Rule::list_by_user(pool, self.user).await?;
        let mut tx_list =
            Transaction::list_by_date(pool, self.account_id, from, to, None, true).await?;
        for tx in tx_list.iter_mut() {
            println!("Checking {}", tx.get_description());
            if tx.recategorize(pool, &rules).await? {
                println!(
                    "Tx {} updated with category {}",
                    tx.get_id(),
                    tx.get_category().unwrap_or(0)
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Account;
    use crate::models::users::User;
    use sqlx::SqlitePool;

    async fn get_db() -> SqlitePool {
        crate::create_db("sqlite://account_test.db").await.unwrap()
    }

    async fn remove_db(pool: SqlitePool) {
        pool.close().await;
        std::fs::remove_file("account_test.db").unwrap();
    }

    async fn new_user(pool: &SqlitePool) -> User {
        User::create_user(pool, "account_test", "pass")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn create_test() {
        let pool = get_db().await;
        let user = new_user(&pool).await;
        Account::new(&pool, user.get_id(), "account_test")
            .await
            .unwrap();
        remove_db(pool).await;
    }
}
