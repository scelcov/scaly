use anyhow::Context;
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = upscale::Cli::parse();
    upscale::run(cli).await.context("upscale exited")
}
