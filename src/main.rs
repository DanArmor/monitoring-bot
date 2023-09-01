use hyper::server::conn::AddrIncoming;
use log::{debug, error, info};
use main_client::MainClient;
use mobot::API;

use mobot::api::SendMessageRequest;
use mobot::handler::{BotState, State};
use serde::de::DeserializeOwned;

use std::sync::Arc;

mod bot_init;
mod config;
mod main_client;

use hyper::{Body, Request, Response, Server, StatusCode};
// Import the routerify prelude traits.
use routerify::prelude::*;
use routerify::{RequestInfo, Router, RouterService};
use std::{convert::Infallible, net::SocketAddr};

#[derive(serde::Deserialize)]
struct AlertRequest {
    pub from: String,
    pub theme: String,
    pub text: String,
}

// Basic error handler
async fn error_handler<S: BotState>(_: Arc<API>, _: i64, _: State<S>, err: anyhow::Error) {
    error!("{}", err);
}

async fn server_error_handler(err: routerify::RouteError, _: RequestInfo) -> Response<Body> {
    error!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong!")))
        .unwrap()
}

async fn get_request_body<T: DeserializeOwned>(req: &mut Request<Body>) -> anyhow::Result<T> {
    match hyper::body::to_bytes(req.body_mut()).await {
        Ok(bytes) => {
            let bytes = bytes.to_vec();
            let body = match serde_json::from_slice::<T>(bytes.as_slice()) {
                Ok(body) => Ok(body),
                Err(e) => Err(anyhow::anyhow!("failed to parse request body: {}", e)),
            };
            body
        }
        Err(e) => Err(anyhow::anyhow!("internal_server_error: {}", e)),
    }
}

async fn fire_handler(mut req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let alert = get_request_body::<AlertRequest>(&mut req).await?;
    let main_client = req.data::<MainClient>().unwrap();

    let admins = main_client.get_admins();
    for admin in admins {
        main_client
            .tg_api
            .send_message(&SendMessageRequest::new(
                admin.to_owned(),
                mobot::api::escape_md(&alert.text),
            ))
            .await?;
    }
    info!(
        "Admins were informed. From: {} / Theme: {}",
        alert.from, alert.theme
    );
    Ok(Response::new(Body::from("Alert done")))
}

fn server_router(main_client: MainClient) -> Router<Body, anyhow::Error> {
    Router::builder()
        .data(main_client)
        .post("/notify/fire", fire_handler)
        .err_handler_with_info(server_error_handler)
        .build()
        .unwrap()
}

async fn notify_users(server: Server<AddrIncoming, RouterService<Body, anyhow::Error>>) {
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err)
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Create bot state
    let bot_state = bot_init::init_app().await?;

    let admins = bot_state.get_admins().clone();
    let addr: SocketAddr = bot_state.get_server_addr().parse().unwrap();

    // Create client for mobot
    let client = mobot::Client::new(bot_state.get_telegram_bot_token());
    let mut router = mobot::Router::<config::MelatoninBotState>::new(client)
        .with_error_handler(error_handler)
        .with_state(bot_state);

    // Create client
    let main_client = main_client::MainClient::new(router.api.clone(), admins);

    info!("Setuped bot router");

    let server_router = server_router(main_client);
    let service = RouterService::new(server_router).unwrap();

    info!("Setuped server router");

    let server = Server::bind(&addr).serve(service);

    tokio::spawn(notify_users(server));
    info!("Server thread was started");
    // Start bot
    info!("Bot was started");
    router.start().await;
    Ok(())
}
