use crate::{
    abi::SignalMessage, config::get_global_config, models::get_global_manager,
    ws_service::ws_handler,
};
use anyhow::Result;
use axum::{
    error_handling::HandleErrorLayer,
    extract::{Json, Query, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{intrinsics::simd::simd_reduce_all, time::Duration};
use tokio::{net::TcpListener, signal, sync::broadcast};
use tokio_tungstenite::connect_async;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

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
    let (tx, _rx) = broadcast::channel(500);

    let app = Router::new()
        .route("/api/v1/add_account", get(add_account))
        .route("/api/v1/get_coin", get(get_coin))
        .route("/api/v1/get_account", get(get_account))
        .route("/api/v1/get_accounts", get(get_accounts))
        .route(
            "/ws",
            get(move |ws: WebSocketUpgrade| ws_handler(ws, tx.clone())),
        )
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
    use_gmgn_check: bool,
}

async fn get_coin(Query(query): Query<TokenQuery>) -> impl IntoResponse {
    let manager = get_global_manager().await.clone();

    match manager.get_coin_with_token(query.token).await {
        Ok(coin) => {
            if coin.is_none() && query.use_gmgn_check {
                // use ws to check
                let url = url::Url::parse(
                    format!("ws://{}/ws", get_global_config().await.host_uri).as_str(),
                )
                .unwrap();

                match connect_async(url.as_str()).await {
                    Ok((ws_stream, _)) => {
                        let (mut ws_sender, ws_receiver) = ws_stream.split();
                        let msg = serde_json::json!({
                            "mint": query.token,
                        })
                        .to_string();

                        if ws_sender
                            .send(tokio_tungstenite::tungstenite::Message::Text(msg))
                            .await
                            .is_err()
                        {
                            return CustomResponse::ok(coin).to_json();
                        }

                        while let Some(Ok(msg)) = ws_receiver.next().await {
                            if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                                let signal: Result<SignalMessage, _> = serde_json::from_str(&text);
                                match signal {
                                    Ok(signal) => {
                                        if signal.mint == query.token && signal.is_suspicious {
                                            warn!(
                                                "check gmgn success, msg: {}, ming: {}",
                                                signal.msg, signal.mint
                                            );
                                            return CustomResponse::ok(Some(query.token)).to_json();
                                        }
                                    }
                                    Err(e) => {
                                        return CustomResponse::err(e.to_string()).to_json();
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        return CustomResponse::err(e.to_string()).to_json();
                    }
                }
            }

            return CustomResponse::ok(coin).to_json();
        }
        Err(e) => CustomResponse::err(e.to_string()).to_json(),
    }
}

#[derive(Deserialize)]
struct AddressQuery {
    address: String,
}

async fn get_account(Query(query): Query<AddressQuery>) -> impl IntoResponse {
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
