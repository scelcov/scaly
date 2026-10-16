# upscale

Local service that queues image and video upscale jobs for an agent.

`upscale mcp` upscales images and videos by 2, 3, or 4 with ffmpeg libplacebo on the GPU. A picture or video pasted in LibreChat is read from the shared `images` and `uploads` folders. Finished files are served at `UPSCALE_PUBLIC_URL/media/<job-id>`, which is a LAN address so a phone can open them.

## Layout

- `crates/upscale-core` — job id, request, in-memory store
- `crates/upscale-http` — Axum routes for a browser or `curl`
- `crates/upscale-mcp` — MCP tools for Claude Code
- `crates/upscale` — one binary, two entry points

## Run

```bash
cargo run -p upscale -- serve
cargo run -p upscale -- mcp
cargo run -p upscale -- mcp --stdio
```

`serve` listens on `127.0.0.1:8787`. `mcp` listens on `127.0.0.1:8788` and `/tmp/upscale-mcp/upscale-mcp.sock`, and serves MCP at `/mcp`. `mcp --stdio` speaks MCP on stdin/stdout. Each process has its own job table.

## Tests and hooks

```bash
cargo test --workspace
cp .githooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

The hook runs `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`.

## LibreChat

One compose stack builds the upscale image and starts LibreChat. The upscale container sees the AMD GPU through `/dev/dri`.

```bash
deploy/librechat/up.sh
```

Open http://localhost:3080 on this machine, or the same host from a phone on the LAN. The theme button in the top-right switches Aubergine and Banana. Under the message box, select the `upscale` MCP server, or add it to an agent under **Add tools**. The first run writes `deploy/librechat/.env` with local secrets and `UPSCALE_PUBLIC_URL`.
