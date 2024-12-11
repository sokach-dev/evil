# Evil

用来跟踪一些邪恶的钱包地址，并记录他的持仓。

## 使用

```bash
# 1. 安装sqlx
cargo install sqlx-cli --no-default-features --features postgres
# 2. 创建数据库
mkdir data
touch data/db.sqlite3
cp env.example .env
sqlx database create
sqlx migrate run
# 3. 编译
cargo build --release
# 4. 运行
cp app.example.toml app.toml
## 4.1 起web服务
./target/release/angel -c app.toml web
## 4.2 启动跟踪
./target/release/angel -c app.toml daemon
```

添加一个地址

```bash
curl "http://127.0.0.1:2211/api/v1/add_account?address=9xHxgDbeQDX51Vof7ruAaYjSYgR87BXRp3ZC62jrmJV1"
{"msg":"ok","data":null}
```

查询一个币地址

```bash
# 有币
curl "http://127.0.0.1:2211/api/v1/get_coin?token=APAkdwfAyqFsQuD92hURMnfUE2dKkjaZjbttx3oZfniy"     
{"msg":"ok","data":{"id":528,"account":"9xHxgDbeQDX51Vof7ruAaYjSYgR87BXRp3ZC62jrmJV1","token":"APAkdwfAyqFsQuD92hURMnfUE2dKkjaZjbttx3oZfniy","created_at":1733293394,"deleted":0}}
# 没币
curl "http://127.0.0.1:2211/api/v1/get_coin?token=APAkdwfAyqFsQuD92hURMnfUE2dKkjaZjbttx3oZfn1y"
{"msg":"ok","data":null}
```

检查一个币是否有特别大的占比
```bash
# 有人占比过大，这里的占比在配置文件里的check_largest_account_hold_coin配置
curl "http://127.0.0.1:2211/api/v1/check_token_largest_accounts?token=4XVHtuLTu35F9vV5JZBNUQGaAZe7KuK8ZQffVssvpump"
{"msg":"ok","data":{"is_suspicion":true} # 表示有人占比过大

curl "http://127.0.0.1:2211/api/v1/check_token_largest_accounts?token=9FABQYprYoaBDjhaqHcQzyMnWzBSYPS3RPLYiTG2pump" 
{"msg":"ok","data":{"is_suspicion":false} # 表示没有人占比过大

# 当你传了一个不存在的token时
url "http://127.0.0.1:2211/api/v1/check_token_largest_accounts?token=9FABQYprYoaBDjhaqHcQzyMnWzBSYPS3RPLYiTG2pum"
{"msg":"get token largest accounts err: RPC response error -32602: Invalid param: could not find mint; ","data":null}
```

## 配置文件
```toml
database_url="sqlite://./data/db.sqlite3"
host_uri="127.0.0.1:2211" # 本地web服务的地址
solana_rpc_url="https://api.mainnet-beta.solana.com" # solana rpc地址逗号分割, 最好替换为自己的如 helius.dev
solana_rpc_curl_interval=10 # 同步关注账户的持仓信息的时间间隔, 单位秒
check_largest_account_hold_coin=100000000.0 # 检查是否有人占比过大的阈值,这里1亿表示如果除了池子有人持币超过1亿就会被标记为可疑
```
