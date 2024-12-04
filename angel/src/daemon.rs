use std::time::Duration;

use crate::solana_rpc::get_tokens_with_account;
use tokio::time::sleep;
use tracing::{error, info};

use crate::{config::get_global_config, models::get_global_manager};

pub async fn daemon() {
    // loop and interval
    let c = get_global_config().await;
    let manager = get_global_manager().await;
    info!("daemon start");
    loop {
        info!("daemon loop, sleep {}s", c.solana_rpc_curl_interval);
        sleep(Duration::from_secs(c.solana_rpc_curl_interval)).await;
        // get evil accounts
        match manager.get_all_accounts().await {
            Ok(accounts) => {
                for account in accounts {
                    match get_tokens_with_account(&account.account, &c.solana_rpc_url).await {
                        Ok(tokens) => {
                            if tokens.len() > 0 {
                                // del old coins
                                if let Err(e) =
                                    manager.del_coin_with_account(&account.account).await
                                {
                                    error!(
                                        "del old coins error: {:?}, account: {}",
                                        e, &account.account
                                    );
                                    continue;
                                }
                            }
                            for token in &tokens {
                                if let Err(e) =
                                    manager.add_new_coin(&account.account, &token.mint).await
                                {
                                    error!(
                                        "add new coin error: {:?}, account: {}, token: {}",
                                        e, &account.account, &token.mint
                                    );
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            error!("get tokens with account error: {:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("get all accounts error: {:?}", e);
            }
        }
    }
}
