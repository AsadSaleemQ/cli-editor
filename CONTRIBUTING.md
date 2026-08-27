# Contributing

## Scope

Changes should preserve native Codex and Claude process fidelity, the signed-manifest boundary, exact PATH rollback, and fail-closed target discovery. Claude Code must remain an unmodified native pass-through.

## Patch workflow

1. Clone `openai/codex` and checkout the commit in `patches/codex/<tag>/upstream.json`.
2. Apply the patch with `git apply --check` followed by `git apply`.
3. Make changes in the pinned worktree.
4. Run the upstream TUI test/fix/format workflow required by its `AGENTS.md`.
5. Regenerate the single distributable patch with `git diff --binary --full-index HEAD -- codex-rs/tui` so lockfile and unrelated upstream changes cannot enter it.
6. Confirm `git apply --reverse --check` on the modified tree and `git apply --check` on a clean pinned tree.
7. Update patch hash, size, verification evidence, and compatibility metadata.

## Dispatcher checks

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets --offline -- -D warnings
cargo test --locked -j 1 --offline
```

Never commit `.work`, `target`, release binaries, private signing keys, user state, PATH snapshots, logs, or credentials. Use conventional purpose-based filenames without authorship markers.

## Pull requests

Explain the user-visible behavior, security boundary, compatibility impact, rollback behavior, and exact tests run. Changes to the patch, manifest schema, trust key, process forwarding, discovery, installation, update, or uninstall logic require both Codex and Claude Code review before release.
