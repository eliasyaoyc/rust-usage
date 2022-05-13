mod noise_codec;
mod pb;

use std::convert::TryInto;
use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use noise_codec::NoiseStream;
use tokio::net::TcpListener;
use tracing::info;

use crate::noise_codec::{NoiseCodec, NOISE_PARAMS};
use crate::pb::request::Command;
use crate::pb::{Request, RequestGet, RequestPut, Response};

#[derive(Debug)]
struct ServerState {
    store: DashMap<String, Vec<u8>>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            store: DashMap::new(),
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
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

    info!("Listening to {:?}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("New client: {:?} accepted", addr);

        let shared = state.clone();

        tokio::spawn(async move {
            // let mut stream = LengthDelimitedCodec::builder()
            //     .length_field_length(2)
            //     .new_framed(stream);

            let mut stream = NoiseCodec::builder(NOISE_PARAMS, false).new_framed(stream)?;
            stream.handshake().await?;

            while let Some(Ok(buf)) = stream.next().await {
                let msg: Request = buf.try_into()?;
                info!("Got a command: {msg:?}");

                let response = match msg.command {
                    Some(Command::Get(RequestGet { key })) => match shared.store.get(&key) {
                        Some(v) => Response::new(key, v.value().to_vec()),
                        None => Response::not_found(key),
                    },
                    Some(Command::Put(RequestPut { key, value })) => {
                        shared.store.insert(key.clone(), value.clone());
                        Response::new(key, value)
                    }
                    None => unimplemented!(),
                };
                stream.send(response.into()).await?;
            }
            Ok::<(), anyhow::Error>(())
        });
    }
}
