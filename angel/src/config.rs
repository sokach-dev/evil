use anyhow::Result;
use std::{env, str::FromStr, sync::Arc};
use tokio::{fs, sync::OnceCell};
use validator::Validate;

#[derive(Clone, Debug, Validate, serde::Deserialize)]
pub struct Config {
    #[validate(length(min = 1))]
    pub database_url: String, // database url
    #[validate(length(min = 1))]
    pub host_uri: String, // web服务端口 0.0.0.0:8080
    #[validate(length(min = 1))]
    pub solana_rpc_url: String, // solana rpc url, split by , eg https://a.com, https://b.com
    #[validate(range(min = 1))]
    pub solana_rpc_curl_interval: u64, // solana rpc curl interval, eg 60 -> 60s

    #[validate(range(min = 100000.0))]
    pub check_largest_account_hold_coin: f64, // 要检查的最大账户持有币种数量,不能大于这个数量
}

impl FromStr for Config {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

pub static GLOBAL_CONFIG: OnceCell<Arc<Config>> = OnceCell::const_new();

pub async fn get_global_config() -> &'static Arc<Config> {
    let config_url = env::var("ANGEL_CONFIG").expect("ANGEL_CONFIG is not set env");

    GLOBAL_CONFIG
        .get_or_init(|| async {
            Arc::new(
                fs::read_to_string(config_url)
                    .await
                    .expect("Failed to read config file")
                    .parse::<Config>()
                    .expect("Failed to parse config"),
            )
        })
        .await
}

impl Config {
    pub fn get_random_solana_rpc_url(&self) -> String {
        let urls: Vec<&str> = self.solana_rpc_url.split(",").collect();
        let index = rand::random::<usize>() % urls.len();
        urls[index].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_random_solana_rpc_url() {
        let config = Config {
            database_url: "test_db".to_string(),
            host_uri: "localhost:8080".to_string(),
            solana_rpc_url: "https://a.com,https://b.com,https://c.com".to_string(),
            solana_rpc_curl_interval: 60,
            check_largest_account_hold_coin: 100000.0,
        };

        let url1 = config.get_random_solana_rpc_url();
        assert!(vec!["https://a.com", "https://b.com", "https://c.com"].contains(&url1.as_str()));

        let url2 = config.get_random_solana_rpc_url();
        assert!(vec!["https://a.com", "https://b.com", "https://c.com"].contains(&url2.as_str()));

        // Test single URL case
        let config_single = Config {
            database_url: "test_db".to_string(),
            host_uri: "localhost:8080".to_string(),
            solana_rpc_url: "https://single.com".to_string(),
            solana_rpc_curl_interval: 60,
            check_largest_account_hold_coin: 100000.0,
        };

        let single_url = config_single.get_random_solana_rpc_url();
        assert_eq!(single_url, "https://single.com");
    }
}
