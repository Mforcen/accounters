use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Result, SqlitePool};

use super::{rules::Rule, transaction::Transaction};

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct Account {
    account_id: i32,
    account_name: String,
}

impl Account {
    pub fn get_id(&self) -> i32 {
        self.account_id
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

    pub async fn new(pool: &SqlitePool, name: &str) -> Result<Self> {
        let row = sqlx::query("INSERT INTO accounts(account_name) VALUES (?) RETURNING *")
            .bind(name)
            .fetch_one(pool)
            .await?;
        Self::from_row(&row)
    }

    pub async fn list(pool: &SqlitePool) -> Result<Vec<Self>> {
        let rows = sqlx::query("SELECT * FROM accounts")
            .fetch_all(pool)
            .await?;
        let mut res = Vec::new();
        for r in &rows {
            res.push(Account::from_row(r)?)
        }
        Ok(res)
    }

    pub async fn recategorize_transactions(&self, pool: &SqlitePool) -> Result<()> {
        let rules = Rule::list(pool).await?;
        let mut tx_list = Transaction::list_uncategorized(pool, self.account_id).await?;
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
    use sqlx::SqlitePool;

    async fn get_db() -> SqlitePool {
        crate::create_db("sqlite://account_test.db").await.unwrap()
    }

    async fn remove_db(pool: SqlitePool) {
        pool.close().await;
        std::fs::remove_file("account_test.db").unwrap();
    }

    #[tokio::test]
    async fn create_test() {
        let pool = get_db().await;
        Account::new(&pool, "account_test").await.unwrap();
        remove_db(pool).await;
    }
}
