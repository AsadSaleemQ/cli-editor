# CLI Editor technical guide

## What ships

The Git repository contains the Rust dispatcher, a compact patch against a pinned Codex commit, compatibility schemas, the VS Code companion source, documentation, and release automation. Upstream source trees, build directories, local validation output, and compiled release binaries remain outside version control. Published Windows artifacts belong in GitHub Releases.

A release bundle contains:

- `cli-editor.exe`: installer, dispatcher, compatibility guard, updater, doctor, and uninstaller.
- `codex-enhanced.exe`: the pinned Codex build with the desktop composer patch.
- `codex-code-mode-host.exe`: the matching upstream code-mode helper.
- `cli-editor-vscode.vsix`: a minimal companion that routes Ctrl+Home and Ctrl+End to the active terminal application instead of VS Code scrollback commands and routes Ctrl+V as text or terminal-native image paste.
- `compatibility-manifest.json` and `.sig`: Ed25519-signed artifact and compatibility metadata.
- `THIRD_PARTY_LICENSES_CLI_EDITOR.html` and `THIRD_PARTY_LICENSES_CODEX.html`: generated dependency license texts for the two Rust binary sets.

## Commands

```text
cli-editor install [--dry-run]
cli-editor status
cli-editor doctor [--json]
cli-editor default codex
cli-editor default claude|all [--strict|--no-strict]
cli-editor restore codex|claude|all
cli-editor update --bundle DIRECTORY
cli-editor rollback [--release RELEASE]
cli-editor repair --adopt-native codex|claude
cli-editor uninstall
cli-editor run codex|claude -- ARGS...
codex cli-editor [-- CODEX_ARGS...]
claude cli-editor [-- CLAUDE_ARGS...]
```

Installation adds one owned shim directory to the beginning of the current user's PATH and broadcasts the Windows environment change. When VS Code is safely discovered beside its official CLI launcher, installation adds the bundled terminal companion and records ownership so uninstall removes only an extension CLI Editor added. VS Code profiles maintain separate extension and keybinding registries, so the companion and its bindings must exist in the profile that owns the terminal workspace. Strict flags apply only to Claude or `all`; `default all` preserves the existing Claude strict setting unless `--strict` or `--no-strict` is explicit. A default request fails without changing state when its native CLI was not discovered; after installing that CLI, `cli-editor repair --adopt-native codex|claude` adds its route.

Before changing PATH, CLI Editor records the exact raw registry value, including its type and expansion text. If PATH is unchanged, uninstall restores that snapshot byte-for-byte. If PATH changed later, uninstall removes only CLI Editor's owned entry and preserves the later edits. A pre-existing shim entry is never claimed or removed. Cleanup is confined to the owned `%LOCALAPPDATA%\CLIEditor` tree and never traverses a reparse-point directory. Self-uninstall removes its running shim from command resolution before completing cleanup and reports any inert Windows-locked residue that must be removed after exit.

## Codex and Claude behavior

The VS Code companion binds Ctrl+Home and Ctrl+End at the terminal host and sends the standard xterm modified-key sequences to the active terminal. This prevents VS Code from consuming those chords as scroll-to-top and scroll-to-bottom before Codex can interpret them. Its Ctrl+V handler delegates non-empty text to VS Code's terminal paste command and sends terminal-native Ctrl+V when clipboard text is empty so image paste can reach the CLI. The companion does not read prompt or transcript content, store clipboard content, or transmit it. User-level keybindings have final precedence, and named profiles keep independent extension and keybinding registries.

Enhanced Codex is selected only by an explicit `codex cli-editor` invocation or an enabled Codex default. A signed cached manifest must support the exact native Codex version. Invalid signatures, rollback sequences, unsupported Codex versions, or expired manifests cannot authorize an enhanced binary. An unlisted VS Code host version produces a visible warning and continues because host drift does not change the pinned Codex binary. A defaulted route otherwise degrades to verified native Codex; an explicit enhanced request fails visibly.

`claude cli-editor` performs signed managed-compatibility validation, then forwards to the user's unchanged native `claude.exe`. The explicit form warns and forwards when the signed validation set is stale or unavailable; users can opt into fail-closed behavior with a strict managed default. Official npm Claude launchers are resolved only through the exact `@anthropic-ai/claude-code` package metadata and its native `bin\claude.exe`; arbitrary `.cmd`, `.bat`, and `.ps1` launchers remain unsupported because safe shell forwarding cannot provide the process-fidelity contract. A safely rejected launcher leaves only that CLI unmanaged and does not block installation for another supported CLI.

A legitimate in-place native update may self-adopt only when the recorded canonical path, package root, expected vendor/package family, and executable shape remain unchanged. Cold native probes have a bounded 60-second budget; after a timeout, identity-validated default Codex and Claude routes warn and forward natively, while explicit enhanced Codex still fails visibly. CLI Editor records completed adoptions with old/new versions and hashes in a bounded journal. Relocation or identity changes require explicit `repair --adopt-native`.

## Updates and rollback

`update --bundle` works for Codex-managed, Claude-only, or dual-CLI installations and verifies the detached Ed25519 signature, monotonic manifest sequence, expiry, minimum dispatcher version, and every artifact's declared size and SHA-256. It stages and smoke-tests enhanced Codex with a 60-second cold-artifact budget before acquiring the state lock. Activation retains the active release and two prior signed releases, replaces cache/shims transactionally, and restores prior files if state publication fails. `cli-editor rollback` re-verifies a retained signed release and activates it without lowering the highest observed manifest sequence.

A dispatcher-changing update must be launched from the new external bundle. This prevents a Windows executable from overwriting itself. Existing sessions retain their already-open executable; locked shims cause update failure and rollback rather than mixed state.

## Build

Requirements: Git, Rust 1.95.0, and the Windows MSVC toolchain. The local GNU/LLVM toolchain can build the patched Codex but Rusty V8 does not publish the matching code-mode-host archive; the release workflow therefore uses `x86_64-pc-windows-msvc`.

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test -j 1 --offline
```

To pass a literal first argument named `cli-editor` to either native CLI, use `codex -- cli-editor` or `claude -- cli-editor`.

The optional `--` immediately after the `cli-editor` control token is a consumed separator. To forward a literal leading delimiter, repeat it, for example `codex cli-editor -- -- help`.

To reproduce the upstream patch manually:

```powershell
git clone https://github.com/openai/codex.git upstream-codex
git -C upstream-codex checkout 3ba0f711642a888aec92a611a3f3b2211157ff89
git -C upstream-codex apply ..\cli-editor\patches\codex\rust-v0.148.0\0001-desktop-composer.patch
cargo build --manifest-path upstream-codex\codex-rs\Cargo.toml --release -j 1 -p codex-cli -p codex-code-mode-host
```

Release dispatches are serialized, and preparation requires the requested manifest sequence to exceed every existing release or draft manifest sequence. GitHub performs two unsigned builds with normalized paths and deterministic timestamps, applies the upstream-maintained deterministic argument-order fix to an exact-hash-verified temporary copy of `i18n-embed-fl` 0.9.4, refuses the candidate unless every artifact hash matches bit-for-bit, and provenance-attests the unsigned executables, VSIX, source archive, and SBOM. The Ed25519 seed corresponding to `compatibility/public-key.hex` remains only on the maintainer workstation. `scripts/publish_draft_release.ps1` accepts a successful release-workflow run ID, requires that run to match local `HEAD`, downloads only its primary candidate, runs the isolated finalizer locally, checks the generated public key, publishes a signed draft, and deletes its temporary `.artifacts` workspace in `finally`. `src/bin/sign_release.rs` signs exact manifest bytes. The signed draft must then pass downloaded-artifact install, bare default and explicit routes for both CLIs, native restoration, uninstall, residue, exact raw-PATH, and latency checks before publication. v0.1 does not include Authenticode signing or established SmartScreen reputation.

## Documentation

- [Verification and current release gates](docs/VERIFICATION.md)
- [Desktop composer behavior](docs/DESKTOP_COMPOSER_BEHAVIOR.md)
- [Updates and rollback](docs/UPDATE_AND_ROLLBACK.md)
- [Security policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)
- [Release notes](RELEASE_NOTES.md)
- [Third-party notices](THIRD_PARTY_NOTICES.md)

## License and trademarks

CLI Editor source is Apache-2.0. The patched Codex files retain upstream licensing and statements of modification. OpenAI, Codex, Anthropic, Claude, Microsoft, and VS Code are trademarks of their respective owners. Their names identify compatibility only.
