# Agent instructions

These rules apply to every change in this repository. They are not optional.

## Rust skills

Before writing, reviewing, or refactoring any Rust, read the rust-skills index and the rule files that match the change.

Skill index:

- `~/.cursor/skills/rust-skills/SKILL.md`
- `~/.claude/skills/rust-skills/SKILL.md`

Open the relevant files under that skill's `rules/` directory. Apply them. Do not rely on memory of the rules. Priority is CRITICAL, then HIGH, then MEDIUM.

## Context7

Before using a library, crate, framework, SDK, or CLI API, query the Context7 MCP server for the current documentation. Do this even for familiar crates such as Tokio, Axum, Serde, rmcp, and Clap.

Use the project MCP server named `context7` (`https://mcp.context7.com/mcp`):

1. Resolve the library id.
2. Query the docs for the exact API you are about to call.
3. Follow the docs you just retrieved. Do not invent signatures, features, or config fields.

If Context7 is unavailable, say so and stop rather than guessing the API.
