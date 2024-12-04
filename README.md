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
