use std::net::SocketAddr;

use anyhow::Context;
use clap::{Parser, Subcommand};
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use upscale_core::JobStore;
use upscale_mcp::UpscaleServer;

#[derive(Debug, Parser)]
#[command(name = "upscale", about = "Queue local image and video upscales")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// HTTP status API on localhost.
    Serve {
        #[arg(long, default_value = "127.0.0.1:8787")]
        addr: SocketAddr,
    },
    /// MCP tools on stdin/stdout for Claude Code.
    Mcp,
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "upscale=info".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    match cli.command {
        Command::Serve { addr } => serve(addr).await,
        Command::Mcp => mcp().await,
    }
}

async fn serve(addr: SocketAddr) -> anyhow::Result<()> {
    let store = JobStore::default();
    let app = upscale_http::router(store);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    tracing::info!(%addr, "http listening");
    axum::serve(listener, app)
        .await
        .context("http server stopped")?;
    Ok(())
}

async fn mcp() -> anyhow::Result<()> {
    let server = UpscaleServer::new(JobStore::default());
    let running = server
        .serve(stdio())
        .await
        .context("failed to start mcp stdio")?;
    running.waiting().await.context("mcp session ended")?;
    Ok(())
}
