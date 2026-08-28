# Verification

Date: 2026-08-27

## Supported baseline

- Platform: Windows terminals, including the integrated VS Code terminal.
- Enhanced Codex: upstream `rust-v0.148.0`, commit `3ba0f711642a888aec92a611a3f3b2211157ff89`.
- Codex patch: 38 files, 105,082 bytes, SHA-256 `09366ede4de32f98d608d960bf87b137ca91691bc27fb25511c30338c10bcaed`.
- Claude Code: native pass-through only; Claude is not patched or redistributed.
- Release toolchain: Rust 1.95.0, Windows MSVC.

Compatibility is signed and exact for enhanced Codex. Unknown or unsupported versions fail visibly for an explicit enhanced request and fall back to verified native Codex for a default route. Native Claude remains available unless the user explicitly enables strict managed validation.

## Continuous validation

The public CI workflow runs these gates on every push and pull request:

- publication-boundary and credential scanning;
- Actionlint 1.7.12 for both GitHub Actions workflows;
- dispatcher formatting and warnings-denied Clippy;
- all 67 dispatcher tests plus the VS Code bridge routing and deterministic VSIX checks, release build, and isolated release-finalizer fixture;
- clean pinned Codex aggregate: 3,557 passed, 26 known Windows snapshot failures, 10 ignored;
- patched Codex aggregate: 3,571 passed, the same 26 failure names, 10 ignored;
- pinned Rusty V8 150.4.0 download integrity and MSVC `codex-code-mode-host` check.

Local pre-publication validation also installed the deterministic VSIX through VS Code's CLI in isolated user-data and extensions directories; VS Code recognized it as `asadsaleemq.cli-editor-vscode-0.1.0`.

The patched suite adds 14 passing tests and no new failure or ignored test. The committed baseline gate checks the complete pass/fail/ignored totals and exact known failure-name set.

The Rusty V8 static archive is cross-checked against the pinned upstream `MODULE.bazel` URL and SHA-256. The generated binding is protected by an exact size and SHA-256 pin; upstream does not publish an equivalent binding hash, so that binding value is a deliberate trust-on-first-use pin rather than an independently cross-referenced provenance claim.

## Reproduce locally

With Rust 1.95 installed:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked -j 1
.\scripts\check_publication_candidate.ps1
```

Patch preflight:

```powershell
git clone https://github.com/openai/codex.git upstream-codex
git -C upstream-codex checkout 3ba0f711642a888aec92a611a3f3b2211157ff89
git -C upstream-codex apply --check .\patches\codex\rust-v0.148.0\0001-desktop-composer.patch
```

## User acceptance and remaining release gates

An earlier live VS Code terminal test confirmed touchpad/wheel scrolling, clipboard copy/paste, image paste, mouse cursor behavior, composer click/drag editing, and Ctrl+X/A/C/V. The later fix that releases capture immediately after a completed turn still requires a fresh live confirmation for both empty-composer transcript selection and non-empty-draft editing.

Before a non-draft release:

1. complete the live mouse-capture and Ctrl+Home/End retest;
2. run the hosted unsigned release workflow, including the first actual V8 link and bit-for-bit rebuild parity;
3. download that exact candidate, sign it locally without uploading the seed, and publish a draft;
4. inspect the signed ZIP, hashes, SBOM, unsigned-build provenance, and generated license reports; and
5. complete downloaded-artifact install/default/restore/update/rollback/uninstall and repeated-sequence acceptance on an isolated Windows account or clean machine.

No signed draft, reproducible-release parity, or downloaded-artifact lifecycle success is claimed until those gates execute.
