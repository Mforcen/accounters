use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Result, Sqlite, SqlitePool};

use md5::{Digest, Md5};

use crate::models::rules::Rule;

pub enum TxConflictResolutionMode {
    Nothing,
    Error,
    Duplicate,
}

#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct Transaction {
    transaction_id: i32,
    account: i32,
    description: String,
    transaction_timestamp: DateTime<Utc>,
    category: Option<i32>,
    amount: i32,
    accumulated: i32,
    #[serde(default, skip_serializing)]
    hash: Option<String>,
}

impl Transaction {
    pub async fn new(
        pool: &SqlitePool,
        account: i32,
        desc: &str,
        ts: &DateTime<Utc>,
        category: Option<i32>,
        amount: i32,
        on_conflict: TxConflictResolutionMode,
    ) -> Result<Self> {
        let hash = Transaction::get_tx_hash(account, &desc, &ts, amount);
        let tx_db = match sqlx::query("SELECT * FROM transactions WHERE hash=? LIMIT 1")
            .bind(&hash)
            .fetch_one(pool)
            .await
        {
            Ok(row) => Some(Transaction::from_row(&row)?),
            Err(sqlx::Error::RowNotFound) => None,
            Err(e) => {
                return Err(e);
            }
        };

        if let Some(tx) = tx_db {
            match on_conflict {
                TxConflictResolutionMode::Nothing => {
                    return Ok(tx);
                }
                TxConflictResolutionMode::Error => {
                    return Err(sqlx::Error::RowNotFound);
                }
                _ => {}
            }
        }

        sqlx::query(concat!(
            "INSERT INTO transactions(",
            "account, description, transaction_timestamp, category, amount, hash",
            ") VALUES (?,?,?,?,?,?) RETURNING *"
        ))
        .bind(account)
        .bind(desc)
        .bind(ts)
        .bind(category)
        .bind(amount)
        .bind(hash)
        .fetch_one(pool)
        .await
        .map(|x| Transaction::from_row(&x).unwrap())
    }

    pub async fn list(
        pool: &SqlitePool,
        account: i32,
        limit: i32,
        offset: i32,
        asc: bool,
    ) -> Result<Vec<Self>> {
        let rows = sqlx::query(
			if asc {
				"SELECT * FROM transactions WHERE account=? ORDER BY transaction_timestamp ASC LIMIT ? OFFSET ?"
			} else {
				"SELECT * FROM transactions WHERE account=? ORDER BY transaction_timestamp DESC LIMIT ? OFFSET ?"
			}
		).bind(account)
         .bind(limit)
         .bind(offset)
         .fetch_all(pool)
         .await?;

        let mut res = Vec::new();
        for r in &rows {
            res.push(Transaction::from_row(r)?);
        }
        Ok(res)
    }

    pub async fn list_by_user(
        pool: &SqlitePool,
        user: i32,
        limit: i32,
        offset: i32,
        asc: bool,
    ) -> Result<Vec<Self>> {
        let rows = sqlx::query(
			if asc {
				"SELECT t.* FROM transactions t JOIN accounts a ON a.account_id=t.account WHERE a.user=? ORDER BY transaction_timestamp ASC LIMIT ? OFFSET ?"
			} else {
				"SELECT t.* FROM transactions t JOIN accounts a ON a.account_id=t.account WHERE a.user=? ORDER BY transaction_timestamp DESC LIMIT ? OFFSET ?"
			}
		).bind(user)
         .bind(limit)
         .bind(offset)
         .fetch_all(pool)
         .await?;

        let mut res = Vec::new();
        for r in &rows {
            res.push(Transaction::from_row(r)?);
        }
        Ok(res)
    }

    pub fn query_by_date<'a>(
        account: i32,
        after: Option<DateTime<Utc>>,
        before: Option<DateTime<Utc>>,
        limit: Option<i32>,
        asc: bool,
    ) -> sqlx::QueryBuilder<'a, Sqlite> {
        let mut query = sqlx::QueryBuilder::new("SELECT * FROM TRANSACTIONS WHERE account=");
        query.push_bind(account);

        if let Some(after) = after {
            query.push(" AND transaction_timestamp >= ");
            query.push_bind(after);
        }

        if let Some(before) = before {
            query.push(" AND transaction_timestamp < ");
            query.push_bind(before);
        }

        if asc {
            query.push(" ORDER BY transaction_timestamp ASC");
        } else {
            query.push(" ORDER BY transaction_timestamp DESC");
        }

        if let Some(lim) = limit {
            query.push(" LIMIT ");
            query.push_bind(lim);
        }

        query
    }

    pub async fn list_by_date(
        pool: &SqlitePool,
        account: i32,
        after: Option<DateTime<Utc>>,
        before: Option<DateTime<Utc>>,
        limit: Option<i32>,
        asc: bool,
    ) -> Result<Vec<Self>> {
        let mut query = Self::query_by_date(account, after, before, limit, asc);

        let rows = query.build().fetch_all(pool).await?;

        let mut res = Vec::new();
        for r in &rows {
            res.push(Transaction::from_row(r)?);
        }
        Ok(res)
    }

    pub fn get_id(&self) -> i32 {
        self.transaction_id
    }

    pub fn get_account(&self) -> i32 {
        self.account
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_timestamp(&self) -> &DateTime<Utc> {
        &self.transaction_timestamp
    }

    pub fn get_category(&self) -> Option<i32> {
        self.category
    }

    pub async fn set_category(&mut self, pool: &SqlitePool, new_category: i32) -> Result<()> {
        sqlx::query("UPDATE transactions SET category=? WHERE transaction_id=?")
            .bind(new_category)
            .bind(self.transaction_id)
            .execute(pool)
            .await?;
        self.category = Some(new_category);
        Ok(())
    }

    pub async fn recategorize(&mut self, pool: &SqlitePool, rules: &Vec<Rule>) -> Result<bool> {
        for r in rules.iter() {
            if r.matches(&self.description)
                .map_err(|_| sqlx::Error::Protocol("RegexError".to_string()))?
            {
                self.set_category(pool, r.category).await?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn get_amount(&self) -> i32 {
        self.amount
    }

    pub async fn set_description(&mut self, pool: &SqlitePool, desc: &str) -> Result<()> {
        sqlx::query("UPDATE transactions SET description=?, hash=? WHERE transaction_id=?")
            .bind(desc)
            .bind(Transaction::get_tx_hash(
                self.account,
                desc,
                &self.transaction_timestamp,
                self.amount,
            ))
            .bind(self.transaction_id)
            .execute(pool)
            .await?;
        self.description = desc.to_string();
        Ok(())
    }

    pub fn get_tx_hash(account: i32, description: &str, ts: &DateTime<Utc>, amount: i32) -> String {
        let mut hasher = Md5::new();
        hasher.update(format!(
            "{}/{}/{}/{}",
            account,
            description,
            ts.to_rfc3339(),
            amount
        ));
        let mut out = String::new();
        out.reserve(32);
        for byte in hasher.finalize().iter() {
            out.push_str(&format!("{:02x?}", byte));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{Transaction, TxConflictResolutionMode};
    use crate::models::{account::Account, users::User};
    use sqlx::SqlitePool;

    async fn get_db() -> SqlitePool {
        crate::create_db("sqlite://tx_test.db").await.unwrap()
    }

    async fn remove_db(pool: SqlitePool) {
        pool.close().await;
        std::fs::remove_file("tx_test.db").unwrap();
    }

    async fn new_user(pool: &SqlitePool) -> User {
        User::create_user(pool, "testuser", "pass").await.unwrap()
    }

    #[tokio::test]
    async fn create_test() {
        let pool = get_db().await;
        let user = new_user(&pool).await;
        let acc = Account::new(&pool, user.get_id(), "tx_test").await.unwrap();
        let tx = Transaction::new(
            &pool,
            acc.get_id(),
            "Test transaction",
            &chrono::Utc::now(),
            None,
            100,
            TxConflictResolutionMode::Nothing,
        )
        .await
        .unwrap();

        println!("{tx:?}");

        remove_db(pool).await;
    }
}
