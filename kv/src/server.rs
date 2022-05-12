mod pb;
mod noise_codec;

use anyhow::Result;
use prost::Message;
use crate::pb::RequestGet;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Debug)]
struct ServerState {
    store: DashMap<String, Vec<u8>>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState { store: DashMap::new() }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        self::new()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let state = Arc::new(ServerState::new());
    let addr = "0.0.0.0:8888";
    let listener = TcpListener::bind(addr).await?;

    info!("Listening to {:?}",addr);

    tokio::spawn(async move {
    });
}
