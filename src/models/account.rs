use chrono::{prelude::*, Duration, DurationRound};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Result, SqlitePool};

use super::{rules::Rule, Transaction};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct AccountSnapshot {
    account: i32,
    datestamp: DateTime<Utc>,
    amount: i32,
}

impl AccountSnapshot {
    pub async fn get(
        pool: &SqlitePool,
        account: i32,
        date: DateTime<Utc>,
    ) -> Result<AccountSnapshot> {
        sqlx::query("SELECT * FROM account_snapshot WHERE account=? AND datestamp=?")
            .bind(account)
            .bind(date)
            .fetch_one(pool)
            .await
            .and_then(|r| AccountSnapshot::from_row(&r))
    }

    pub async fn get_last(
        pool: &SqlitePool,
        account: i32,
        date: DateTime<Utc>,
    ) -> Result<AccountSnapshot> {
        sqlx::query("SELECT * FROM account_snapshot WHERE account=? AND datestamp<=? LIMIT 1")
            .bind(account)
            .bind(date)
            .fetch_one(pool)
            .await
            .and_then(|r| AccountSnapshot::from_row(&r))
    }

    pub async fn list(
        pool: &SqlitePool,
        account: i32,
        limit: Option<i32>,
        offset: Option<i32>,
        asc: bool,
    ) -> sqlx::Result<Vec<AccountSnapshot>> {
        let mut query = sqlx::QueryBuilder::new("SELECT * FROM account_snapshot WHERE account=");
        query.push_bind(account);

        if let Some(limit) = limit {
            query.push(" LIMIT ");
            query.push_bind(limit);
        }

        if let Some(offset) = offset {
            query.push(" OFFSET ");
            query.push_bind(offset);
        }

        if asc {
            query.push(" ORDER BY datestamp ASC");
        } else {
            query.push(" ORDER BY datestamp DESC");
        }

        let rows = query.build().fetch_all(pool).await?;

        let mut res = Vec::new();
        for r in rows.iter() {
            res.push(AccountSnapshot::from_row(r)?);
        }
        Ok(res)
    }

    pub async fn list_by_date(
        pool: &SqlitePool,
        account: i32,
        after: Option<DateTime<Utc>>,
        before: Option<DateTime<Utc>>,
        limit: Option<i32>,
        asc: bool,
    ) -> sqlx::Result<Vec<AccountSnapshot>> {
        let mut query = sqlx::QueryBuilder::new("SELECT * FROM account_snapshot WHERE account=");
        query.push_bind(account);

        if let Some(after) = after {
            query.push(" AND datestamp >= ");
            query.push_bind(after);
        }

        if let Some(before) = before {
            query.push(" AND datestamp < ");
            query.push_bind(before);
        }

        if let Some(limit) = limit {
            query.push(" LIMIT ");
            query.push_bind(limit);
        }

        if asc {
            query.push(" ORDER BY datestamp ASC");
        } else {
            query.push(" ORDER BY datestamp DESC");
        }

        let rows = query.build().fetch_all(pool).await?;

        let mut res = Vec::new();
        for r in rows.iter() {
            res.push(AccountSnapshot::from_row(r)?);
        }
        Ok(res)
    }

    pub async fn delete_by_dates(
        pool: &SqlitePool,
        account: i32,
        after: Option<DateTime<Utc>>,
        before: Option<DateTime<Utc>>,
    ) -> sqlx::Result<()> {
        if after.is_none() && before.is_none() {
            return Err(sqlx::Error::RowNotFound);
        }

        let mut query = sqlx::QueryBuilder::new("DELETE FROM account_snapshot WHERE account=");
        query.push_bind(account);

        if let Some(after) = after {
            query.push(" AND datestamp >= ");
            query.push_bind(after);
        }

        if let Some(before) = before {
            query.push(" AND datestamp < ");
            query.push_bind(before);
        }

        query.build().execute(pool).await?;

        Ok(())
    }

    pub async fn insert(&self, pool: &SqlitePool) -> sqlx::Result<()> {
        sqlx::query("INSERT INTO account_snapshot(account, datestamp, amount) VALUES(?,?,?)")
            .bind(self.account)
            .bind(self.datestamp)
            .bind(self.amount)
            .execute(pool)
            .await
            .map(|_| ())
    }

    pub async fn get_next(&self, pool: &SqlitePool) -> sqlx::Result<Option<AccountSnapshot>> {
        let date_next = match Transaction::list_by_date(
            pool,
            self.account,
            Some(self.datestamp + Duration::days(1)),
            None,
            Some(1),
            true,
        )
        .await?
        .first()
        {
            Some(tx) => tx.get_timestamp(),
            None => {
                return Ok(None);
            }
        }
        .duration_trunc(chrono::Duration::days(1))
        .unwrap();

        println!(
            "Starting date: {:?}, ending date: {:?}",
            self.datestamp, date_next
        );

        let tx_list = Transaction::list_by_date(
            pool,
            self.account,
            Some(self.datestamp),
            Some(date_next),
            None,
            true,
        )
        .await?;

        Ok(Some(AccountSnapshot {
            datestamp: date_next,
            account: self.account,
            amount: self.amount + tx_list.iter().fold(0, |acc, tx| acc + tx.get_amount()),
        }))
    }
}

#[derive(FromRow, Serialize, Deserialize)]
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

    pub async fn recalculate_snapshots(
        &self,
        pool: &SqlitePool,
        from: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let mut snap = match from {
            Some(f) => {
                let snapshot = AccountSnapshot::list_by_date(
                    pool,
                    self.get_id(),
                    None,
                    Some(f),
                    Some(1),
                    true,
                )
                .await?;

                if snapshot.is_empty() {
                    AccountSnapshot {
                        account: self.account_id,
                        datestamp: Utc.timestamp_opt(0, 0).unwrap(),
                        amount: 0,
                    }
                } else {
                    snapshot.first().unwrap().clone()
                }
            }
            None => AccountSnapshot {
                account: self.account_id,
                datestamp: Utc.timestamp_opt(0, 0).unwrap(),
                amount: 0,
            },
        };

        AccountSnapshot::delete_by_dates(
            pool,
            self.get_id(),
            Some(snap.datestamp + Duration::hours(12)),
            None,
        )
        .await?;

        while let Some(next) = snap.get_next(pool).await? {
            next.insert(pool).await?;
            snap = next;
        }
        Ok(())
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
