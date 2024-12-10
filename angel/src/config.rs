use anyhow::Result;
use rand::Rng;
use std::{env, str::FromStr, sync::Arc};
use tokio::{fs, sync::OnceCell};
use validator::Validate;

#[derive(Clone, Debug, Default, Validate, serde::Deserialize)]
pub struct Config {
    #[validate(length(min = 1))]
    pub database_url: String, // database url
    #[validate(length(min = 1))]
    pub host_uri: String, // web服务端口 0.0.0.0:8080
    #[validate(length(min = 1))]
    pub solana_rpc_url: String, // solana rpc url
    #[validate(range(min = 1))]
    pub solana_rpc_curl_interval: u64, // solana rpc curl interval, eg 60 -> 60s
    pub use_gmgn_ws_check: bool,
    pub gmgn_ws_url: Option<String>,
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
        let mut rng = rand::thread_rng();

        let index = rng.gen_range(0..urls.len());
        urls[index].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_random_solana_rpc_url() {
        // Create a Config instance with multiple URLs
        let config = Config {
            solana_rpc_url: "http://url1.com,http://url2.com,http://url3.com".to_string(),
            // Initialize other fields if necessary
            ..Default::default()
        };

        // Get the list of URLs
        let urls: Vec<&str> = config.solana_rpc_url.split(',').collect();

        // Call the function
        let random_url = config.get_random_solana_rpc_url();
        println!("random_url: {}", random_url);

        // Assert that the returned URL is in the list
        assert!(urls.contains(&random_url.as_str()));
    }
}
