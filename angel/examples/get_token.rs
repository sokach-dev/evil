use angel::solana_rpc::get_tokens_with_account;

#[tokio::main]
async fn main() {
    let account = "JDLGDgY7jSGkmmRPzQcYtwLQpkrqMgYqY7cFkNb81NTq"; // 当前要有持仓才行
    let solana_rpc_url = "https://api.mainnet-beta.solana.com";
    match get_tokens_with_account(account, solana_rpc_url).await {
        Ok(tokens) => {
            for token in tokens {
                println!("111 {:?}", token);
            }
        }
        Err(e) => {
            println!("111 {:?}", e);
        }
    }
}
