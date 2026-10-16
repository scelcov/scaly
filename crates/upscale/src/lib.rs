mod media;

use std::fs;
use std::net::SocketAddr;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use axum::routing::get;
use clap::{Parser, Subcommand};
use rmcp::ServiceExt;
use rmcp::transport::stdio;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
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
    /// MCP tools. HTTP at `/mcp` by default; `--stdio` for a local client.
    Mcp {
        /// Speak MCP on stdin/stdout instead of HTTP.
        #[arg(long)]
        stdio: bool,
        /// TCP address for clients on this machine. Use `0.0.0.0:8788` inside Docker.
        #[arg(long, default_value = "127.0.0.1:8788")]
        addr: SocketAddr,
        /// Browser-facing origin for finished images and videos. Phones cannot open 127.0.0.1.
        /// `UPSCALE_PUBLIC_URL` is used when this flag is omitted.
        #[arg(long)]
        public_url: Option<String>,
        /// Socket the LibreChat bridge dials. The container cannot open a host TCP port.
        #[arg(long, default_value = "/tmp/upscale-mcp/upscale-mcp.sock")]
        socket: PathBuf,
    },
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    init_tracing();

    match cli.command {
        Command::Serve { addr } => serve(addr).await,
        Command::Mcp {
            stdio,
            addr,
            public_url,
            socket,
        } => {
            if stdio {
                mcp_stdio().await
            } else {
                let from_env = std::env::var("UPSCALE_PUBLIC_URL").ok();
                let public_url = public_url.or(from_env);
                mcp_http(addr, public_url.as_deref(), &socket).await
            }
        }
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

async fn mcp_stdio() -> anyhow::Result<()> {
    tracing::info!(
        "mcp ready on stdin/stdout; logs stay on stderr. Set RUST_LOG=debug for library traces"
    );
    let mut server = UpscaleServer::new(JobStore::default());
    if let Some(inbox) = chat_inbox() {
        server = server.with_inbox(inbox);
    }
    let running = server
        .serve(stdio())
        .await
        .context("failed to start mcp stdio")?;
    tracing::info!("mcp session started");
    running.waiting().await.context("mcp session ended")?;
    tracing::info!("mcp session ended");
    Ok(())
}

async fn mcp_http(
    addr: SocketAddr,
    public_url: Option<&str>,
    socket: &std::path::Path,
) -> anyhow::Result<()> {
    let store = JobStore::default();
    let media_base = media_base_url(addr, public_url);
    let mut hosts = vec![
        "localhost".to_owned(),
        "127.0.0.1".to_owned(),
        "::1".to_owned(),
        "upscale".to_owned(),
    ];
    if let Some(host) = host_of(&media_base) {
        hosts.push(host);
    }
    let config = StreamableHttpServerConfig::default().with_allowed_hosts(hosts);
    let service = StreamableHttpService::<UpscaleServer, LocalSessionManager>::new(
        {
            let store = store.clone();
            let media_base = media_base.clone();
            let inbox = chat_inbox();
            move || {
                let mut server =
                    UpscaleServer::new(store.clone()).with_media_base(media_base.clone());
                if let Some(inbox) = inbox.clone() {
                    server = server.with_inbox(inbox);
                }
                Ok(server)
            }
        },
        Arc::new(LocalSessionManager::default()),
        config,
    );
    let app = axum::Router::new()
        .route("/media/{id}", get(media::file))
        .route("/media/{id}/poster", get(media::poster))
        .with_state(store)
        .nest_service("/mcp", service);
    let tcp = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    if let Some(dir) = socket.parent() {
        fs::create_dir_all(dir).with_context(|| format!("failed to create {}", dir.display()))?;
    }
    match fs::remove_file(socket) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            return Err(err).with_context(|| format!("failed to replace {}", socket.display()));
        }
    }
    let unix = tokio::net::UnixListener::bind(socket)
        .with_context(|| format!("failed to bind {}", socket.display()))?;
    fs::set_permissions(socket, fs::Permissions::from_mode(0o666))
        .with_context(|| format!("failed to chmod {}", socket.display()))?;
    tracing::info!(
        %addr,
        socket = %socket.display(),
        inbox = chat_inbox().map(|path| path.display().to_string()).unwrap_or_default(),
        "mcp listening on /mcp"
    );
    let tcp_app = app.clone();
    tokio::try_join!(
        async move {
            axum::serve(tcp, tcp_app)
                .await
                .context("mcp tcp server stopped")
        },
        async move {
            axum::serve(unix, app)
                .await
                .context("mcp socket server stopped")
        },
    )?;
    Ok(())
}

fn media_base_url(addr: SocketAddr, public_url: Option<&str>) -> String {
    if let Some(url) = public_url.map(str::trim).filter(|url| !url.is_empty()) {
        return url.trim_end_matches('/').to_owned();
    }
    if addr.ip().is_unspecified() {
        format!("http://127.0.0.1:{}", addr.port())
    } else {
        format!("http://{addr}")
    }
}

fn host_of(url: &str) -> Option<String> {
    let rest = url.split("://").nth(1)?;
    let host = rest.split(['/', ':']).next()?.trim();
    if host.is_empty() {
        None
    } else {
        Some(host.to_owned())
    }
}

fn chat_inbox() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("UPSCALE_INBOX") {
        let path = PathBuf::from(path);
        if path.is_dir() {
            return Some(path);
        }
    }
    [
        PathBuf::from("deploy/librechat/images"),
        PathBuf::from("/home/eugene/upscale/deploy/librechat/images"),
    ]
    .into_iter()
    .find(|path| path.is_dir())
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::fmt::format::FmtSpan;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        "info,upscale=info,upscale_mcp=info,upscale_http=info,upscale_core=info"
            .parse()
            .expect("built-in log filter is valid")
    });
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .init();
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;

    use super::media_base_url;

    #[test]
    fn public_url_is_what_a_phone_opens() {
        let addr: SocketAddr = "0.0.0.0:8788".parse().expect("addr");
        assert_eq!(
            media_base_url(addr, Some("http://192.168.1.20:8788/")),
            "http://192.168.1.20:8788"
        );
    }

    #[test]
    fn unspecified_bind_does_not_advertise_zero_address() {
        let addr: SocketAddr = "0.0.0.0:8788".parse().expect("addr");
        assert_eq!(media_base_url(addr, None), "http://127.0.0.1:8788");
    }
}
