use anyhow::Result;
use serde::{Deserialize, Serialize};
use solana_account_decoder::UiAccountData;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

pub type TokenAccounts = Vec<TokenAccount>;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenAccount {
    pub pubkey: String,
    pub mint: String,
    pub amount: String,
    pub ui_amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Amount {
    amount: String,
    decimals: u8,
    ui_amount: f64,
    ui_amount_string: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TokenInfo {
    is_native: bool,
    mint: String,
    owner: String,
    state: String,
    token_amount: Amount,
}

#[derive(Debug, Serialize, Deserialize)]
struct Parsed {
    info: TokenInfo,
    #[serde(rename = "type")]
    account_type: String,
}

// rpc https://solana.com/docs/rpc/http/gettokenaccountsbyowner
pub async fn get_tokens_with_account(account: &str, rpc_url: &str) -> Result<Vec<TokenAccount>> {
    let client = RpcClient::new(rpc_url);

    let account_pubkey = Pubkey::from_str_const(account);
    let token_accounts = client.get_token_accounts_by_owner(
        &account_pubkey,
        solana_client::rpc_request::TokenAccountsFilter::ProgramId(spl_token::id()),
    )?;

    let mut accounts: TokenAccounts = vec![];
    for token_account in token_accounts {
        let account_data = token_account.account.data;
        match account_data {
            UiAccountData::Json(parsed_account) => {
                let parsed: Parsed = serde_json::from_value(parsed_account.parsed)?;
                accounts.push(TokenAccount {
                    pubkey: token_account.pubkey.to_string(),
                    mint: parsed.info.mint,
                    amount: parsed.info.token_amount.amount,
                    ui_amount: parsed.info.token_amount.ui_amount,
                });
            }
            UiAccountData::LegacyBinary(_) | UiAccountData::Binary(_, _) => {
                continue;
            }
        }
    }
    Ok(accounts)
}
