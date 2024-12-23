use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[derive(Serialize, Deserialize)]
struct Config {
    listen_addr: String,
    upstream_addr: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = Arc::new(resolve_config());

    info!("Upstream: {}", config.upstream_addr);
    info!("Listen: {}", config.listen_addr);

    let listener = TcpListener::bind(&config.listen_addr).await?;
    loop {
        let (client, addr) = listener.accept().await?;
        let config_cloned: Arc<Config> = Arc::clone(&config);
        info!("Accept connection {}", addr);
        tokio::spawn(async move {
            let upstream = TcpStream::connect(&config_cloned.upstream_addr).await?;
            proxy(client, upstream).await?;
            Ok::<(), anyhow::Error>(())
        });
    }
}

fn resolve_config() -> Config {
    Config {
        listen_addr: "0.0.0.0:8881".to_string(),
        upstream_addr: "0.0.0.0:8882".to_string(),
    }
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> Result<()> {
    let (mut client_reader, mut client_writer) = client.split();
    let (mut upstream_reader, mut upstream_writer) = upstream.split();

    let client_to_upstream = tokio::io::copy(&mut client_reader, &mut upstream_writer);
    let upstream_to_client = tokio::io::copy(&mut upstream_reader, &mut client_writer);

    if let Err(e) = tokio::try_join!(client_to_upstream, upstream_to_client) {
        warn!("{:?}", e);
    }
    Ok(())
}
