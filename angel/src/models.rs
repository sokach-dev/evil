use std::sync::Arc;

use anyhow::Result;
use sqlx::SqlitePool;
use tokio::sync::OnceCell;

use crate::config::get_global_config;

pub struct ModelsManager {
    pool: SqlitePool,
}
impl ModelsManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

pub static GLOBAL_MANAGER: OnceCell<Arc<ModelsManager>> = OnceCell::const_new();

pub async fn get_global_manager() -> &'static Arc<ModelsManager> {
    GLOBAL_MANAGER
        .get_or_init(|| async {
            let config = get_global_config().await;
            let pool = SqlitePool::connect(&config.database_url)
                .await
                .expect("Failed to connect to database");

            Arc::new(ModelsManager::new(pool))
        })
        .await
}

#[derive(Debug, sqlx::FromRow)]
pub struct Coin {
    pub id: i64,
    pub account_id: i64,
    pub token: String,
    pub created_at: i64,
    pub deleted: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Account {
    pub id: i64,
    pub account: String,
    pub created_at: i64,
    pub deleted: i64,
}

impl ModelsManager {
    pub async fn add_new_account(self, mint: String) -> Result<Account> {
        // judge if the account exists
        let sql_str = format!(
            "SELECT * FROM account WHERE account = '{}' AND DELETED = 0;",
            mint
        );
        let account = sqlx::query_as::<_, Account>(&sql_str)
            .fetch_one(&self.pool)
            .await
            .ok();
        if account.is_some() {
            return Ok(account.unwrap());
        }

        // insert new account
        let sql_str = format!(
            "INSERT INTO account (account, created_at, deleted) VALUES ('{}', {}, 0);",
            mint,
            chrono::Local::now().timestamp()
        );
        let account = sqlx::query_as::<_, Account>(&sql_str)
            .fetch_one(&self.pool)
            .await
            .expect("Failed to insert account");

        Ok(account)
    }

    pub async fn get_account_with_mint(self, mint: String) -> Result<Account> {
        let sql_str = format!(
            "SELECT * FROM account WHERE account = '{}' AND DELETED = 0;",
            mint
        );
        let account = sqlx::query_as::<_, Account>(&sql_str)
            .fetch_one(&self.pool)
            .await
            .expect("Failed to get account");

        Ok(account)
    }

    pub async fn add_new_coin(self, account_id: i64, token: String) -> Result<Coin> {
        // judge if the coin exists
        let sql_str = format!(
            "SELECT * FROM coin WHERE account_id = {} AND token = '{}' AND DELETED = 0;",
            account_id, token
        );
        let coin = sqlx::query_as::<_, Coin>(&sql_str)
            .fetch_one(&self.pool)
            .await
            .ok();
        if coin.is_some() {
            return Ok(coin.unwrap());
        }

        // insert new coin
        let sql_str = format!(
            "INSERT INTO coin (account_id, token, created_at, deleted) VALUES ({}, '{}', {}, 0);",
            account_id,
            token,
            chrono::Local::now().timestamp()
        );
        let coin = sqlx::query_as::<_, Coin>(&sql_str)
            .fetch_one(&self.pool)
            .await
            .expect("Failed to insert coin");

        Ok(coin)
    }

    pub async fn get_coin_with_token(self, token: String) -> Result<Coin> {
        let sql_str = format!(
            "SELECT * FROM coin WHERE token = '{}' AND DELETED = 0;",
            token
        );
        let coin = sqlx::query_as::<_, Coin>(&sql_str)
            .fetch_one(&self.pool)
            .await
            .expect("Failed to get coin");

        Ok(coin)
    }
}
