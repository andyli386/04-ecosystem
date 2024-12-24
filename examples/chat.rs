use anyhow::Result;
use dashmap::DashMap;
use futures::{stream::SplitStream, SinkExt as _, StreamExt};
use std::{fmt, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc,
};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

const MAX_MESSAGE: usize = 128;

#[derive(Debug, Default)]
struct State {
    peers: DashMap<SocketAddr, mpsc::Sender<Arc<Message>>>,
}

#[derive(Debug)]
enum Message {
    UserJoined(String),
    UserLeft(String),
    Chat { sender: String, content: String },
}

#[derive(Debug)]
struct Peer {
    username: String,
    stream: SplitStream<Framed<TcpStream, LinesCodec>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    // console_subscriber::init();

    let addr = "0.0.0.0:8880";
    let listener = TcpListener::bind(addr).await?;
    info!("Listener on {}", addr);

    let state = Arc::new(State::default());

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Accept connection from: {}", addr);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_client(state_clone, addr, stream).await {
                warn!("Failed to handle client addr {} : {}", addr, e);
            }
        });
    }
    // Ok(())
}

async fn handle_client(state: Arc<State>, addr: SocketAddr, stream: TcpStream) -> Result<()> {
    let mut stream = Framed::new(stream, LinesCodec::new());
    stream.send("Enter your username:").await?;

    let username = match stream.next().await {
        Some(Ok(username)) => username,
        Some(Err(e)) => return Err(e.into()),
        None => return Ok(()),
    };

    let mut peer = state.add(addr, username.clone(), stream).await;
    let message = Arc::new(Message::user_joined(&username));
    info!("{}", message);

    state.broadcast(addr, message).await;

    while let Some(line) = peer.stream.next().await {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                warn!("Failed to read line from {}: {}", addr, e);
                break;
            }
        };
        let message = Arc::new(Message::chat(&peer.username, &line));
        info!("{}", message);
        state.broadcast(addr, message).await;
    }

    state.peers.remove(&addr);

    let message = Arc::new(Message::user_left(username));
    info!("{}", message);
    state.broadcast(addr, message).await;

    Ok(())
}

impl State {
    async fn broadcast(&self, addr: SocketAddr, message: Arc<Message>) {
        for peer in self.peers.iter() {
            if peer.key() == &addr {
                continue;
            }

            if let Err(e) = peer.value().send(message.clone()).await {
                warn!("Failed to send message to {}: {}", peer.key(), e);
                self.peers.remove(peer.key());
            };
        }
    }

    async fn add(
        &self,
        addr: SocketAddr,
        username: String,
        stream: Framed<TcpStream, LinesCodec>,
    ) -> Peer {
        let (tx, mut rx) = mpsc::channel(MAX_MESSAGE);
        self.peers.insert(addr, tx);

        let (mut stream_sender, stream_receiver) = stream.split();

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = stream_sender.send(message.to_string()).await {
                    warn!("Failed to send message to {}: {}", addr, e);
                    break;
                }
            }
        });

        Peer {
            username,
            stream: stream_receiver,
        }
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Message::UserJoined(content) => write!(f, "[{}] :)", content),
            Message::UserLeft(content) => write!(f, "[{} :(]", content),
            Message::Chat { sender, content } => write!(f, "{}: {}", sender, content),
        }
    }
}

impl Message {
    fn user_joined(username: impl Into<String>) -> Self {
        let content = format!("{} has joined the chat", username.into());
        Self::UserJoined(content.to_string())
    }
    fn user_left(username: impl Into<String>) -> Self {
        let content = format!("{} has left the chat", username.into());
        Self::UserLeft(content.to_string())
    }

    fn chat(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self::Chat {
            sender: sender.into(),
            content: content.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
