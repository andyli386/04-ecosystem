use std::time::Duration;

use axum::{routing::get, Router};
use tokio::{
    net::TcpListener,
    time::{sleep, Instant},
};
use tracing::{info, instrument, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appendeer = tracing_appender::rolling::minutely("/var/log/mylogs/", "ecosystem.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appendeer);
    let console = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_filter(LevelFilter::INFO);

    let file = fmt::Layer::new()
        .with_span_events(FmtSpan::CLOSE)
        .pretty()
        .with_writer(non_blocking)
        .with_filter(LevelFilter::WARN);

    tracing_subscriber::registry()
        .with(console)
        .with(file)
        .init();

    let addr = "0.0.0.0:9527";
    let app = Router::new().route("/", get(index_handler));

    let listener = TcpListener::bind(addr).await?;
    info!("Starting server on {}", addr);
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[instrument]
async fn index_handler() -> &'static str {
    sleep(Duration::from_millis(10)).await;
    let ret = long_task().await;
    info!(http.status = 200, "index_handler completed");
    ret
}

#[instrument]
async fn long_task() -> &'static str {
    let start = Instant::now();
    sleep(Duration::from_millis(112)).await;
    let elapsed = start.elapsed().as_millis() as u64;
    warn!(app.task_duration = elapsed, "long task completed");
    "Hello World!"
}
