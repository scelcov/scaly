# upscale

Local service that queues image and video upscale jobs for an agent.

The GPU pipeline is not connected. Jobs are stored in memory and stay `queued`.

## Layout

- `crates/upscale-core` — job id, request, in-memory store
- `crates/upscale-http` — Axum routes for a browser or `curl`
- `crates/upscale-mcp` — MCP tools for Claude Code
- `crates/upscale` — one binary, two entry points

## Run

```bash
cargo run -p upscale -- serve
cargo run -p upscale -- mcp
```

`serve` listens on `127.0.0.1:8787`. `mcp` speaks MCP on stdin/stdout. Each process has its own job table.

## Tests and hooks

```bash
cargo test --workspace
cp .githooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

The hook runs `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`.
