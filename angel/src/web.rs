use crate::{config::get_global_config, models::get_global_manager};
use anyhow::Result;
use axum::{
    error_handling::HandleErrorLayer,
    extract::{Json, Query},
    response::IntoResponse,
    routing::get,
    Router,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::{net::TcpListener, signal};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(Debug, Clone, Serialize)]
struct CustomResponse<T> {
    msg: String,
    data: Option<T>,
}

#[allow(dead_code)]
impl<T> CustomResponse<T> {
    fn new(msg: String, data: Option<T>) -> Self {
        Self { msg, data }
    }
    fn ok(data: Option<T>) -> Self {
        CustomResponse {
            msg: "ok".to_string(),
            data,
        }
    }
    fn err(msg: String) -> Self {
        CustomResponse { msg, data: None }
    }
    fn to_json(self) -> Json<CustomResponse<T>> {
        Json(self)
    }
}

pub async fn start_server() -> Result<()> {
    let c = get_global_config().await;
    let app = Router::new()
        .route("/api/v1/add_account", get(add_account))
        .route("/api/v1/get_coin", get(get_coin))
        .route("/api/v1/get_account", get(get_account))
        .route("/api/v1/get_accounts", get(get_accounts))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(30))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        );

    let addr = TcpListener::bind(&c.host_uri).await.unwrap();
    info!("Starting web server at {}", addr.local_addr()?);
    info!("add account: /api/v1/add_account?address=xxx");
    info!("get coin: /api/v1/get_coin?token=xxx");
    info!("get account: /api/v1/get_account?address=xxx");
    info!("get accounts: /api/v1/get_accounts");
    axum::serve(addr, app)
        .with_graceful_shutdown(shoutdown_signal())
        .await
        .unwrap();

    Ok(())
}

#[derive(Deserialize)]
struct AddAccount {
    address: String,
}

async fn add_account(input: Query<AddAccount>) -> impl IntoResponse {
    if input.address.len() != 44 {
        return CustomResponse::err("address length is not 44".to_string()).to_json();
    }

    let manager = get_global_manager().await;
    if let Err(e) = manager.add_new_account(input.address.clone()).await {
        return CustomResponse::err(e.to_string()).to_json();
    }

    CustomResponse::<i32>::ok(None).to_json()
}

#[derive(Deserialize)]
struct TokenQuery {
    token: String,
}

async fn get_coin(Query(query): Query<TokenQuery>) -> impl IntoResponse {
    if query.token.len() != 44 {
        return CustomResponse::err("token length is not 44".to_string()).to_json();
    }
    let manager = get_global_manager().await.clone();

    match manager.get_coin_with_token(query.token).await {
        Ok(coin) => CustomResponse::ok(Some(coin)).to_json(),
        Err(e) => CustomResponse::err(e.to_string()).to_json(),
    }
}

#[derive(Deserialize)]
struct AddressQuery {
    address: String,
}

async fn get_account(Query(query): Query<AddressQuery>) -> impl IntoResponse {
    if query.address.len() != 44 {
        return CustomResponse::err("address length is not 44".to_string()).to_json();
    }
    let manager = get_global_manager().await.clone();

    match manager.get_account_with_mint(query.address).await {
        Ok(account) => CustomResponse::ok(Some(account)).to_json(),
        Err(_) => CustomResponse::err("account not found".to_string()).to_json(),
    }
}

async fn get_accounts() -> impl IntoResponse {
    let manager = get_global_manager().await.clone();
    match manager.get_all_accounts().await {
        Ok(accounts) => CustomResponse::ok(Some(accounts)).to_json(),
        Err(e) => CustomResponse::err(e.to_string()).to_json(),
    }
}

async fn shoutdown_signal() {
    let ctl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl-c signal");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to listen for terminate signal")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctl_c => {},
        _ = terminate => {},
    }

    info!("signal received, shutting down");
}
