use regex::Regex;
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

#[derive(FromRow, Serialize)]
pub struct Rule {
    pub rule_id: i32,
    pub user: i32,
    pub regex: String,
    pub category: i32,
}

impl Rule {
    pub async fn get_by_id(pool: &SqlitePool, rule_id: i32) -> sqlx::Result<Self> {
        sqlx::query("SELECT * FROM rules WHERE rule_id=?")
            .bind(rule_id)
            .fetch_one(pool)
            .await
            .and_then(|r| Rule::from_row(&r))
    }

    pub async fn list_by_user(pool: &SqlitePool, user: i32) -> sqlx::Result<Vec<Self>> {
        let mut res = Vec::new();
        for r in sqlx::query("SELECT * FROM rules WHERE user=?")
            .bind(user)
            .fetch_all(pool)
            .await?
            .iter()
        {
            res.push(Rule::from_row(r)?);
        }
        Ok(res)
    }

    pub async fn new(
        pool: &SqlitePool,
        user: i32,
        regex: String,
        category: i32,
    ) -> sqlx::Result<Self> {
        sqlx::query("INSERT INTO rules(user, regex, category) VALUES (?,?,?) RETURNING *")
            .bind(user)
            .bind(regex)
            .bind(category)
            .fetch_one(pool)
            .await
            .and_then(|r| Rule::from_row(&r))
    }

    pub fn matches(&self, description: &str) -> Result<bool, regex::Error> {
        let re = Regex::new(&self.regex)?;
        Ok(re.is_match(description))
    }
}
