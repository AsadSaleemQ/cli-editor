# CLI Editor engineering case study

## Problem

Codex's terminal composer did not provide the desktop-style mouse, selection, clipboard, and touchpad behavior expected in VS Code. The original working prototype was over 1 GB because it contained an upstream checkout and build outputs, while the reusable change itself was a source patch plus a small launcher.

The public solution also had to preserve Claude Code unchanged, survive CLI and VS Code updates safely, support opt-in and default modes, and uninstall without overwriting later user PATH changes.

## Solution

CLI Editor separates the product into two controlled layers:

- a pinned Codex TUI patch for mouse capture, scrollback handoff, click placement, drag selection, clipboard editing, image paste, and undo/redo; and
- a Rust dispatcher that discovers exact native CLI installations, validates signed compatibility metadata, selects enhanced or native execution, forwards arguments and process controls, and owns transactional install/update/rollback/uninstall state.

Claude Code is neither patched nor redistributed. Official native and npm installations resolve to their native executable through exact package identity checks, then run as a managed native pass-through.

## Safety and lifecycle engineering

The implementation uses detached Ed25519 manifests, monotonic sequence enforcement, expiry and grace handling, artifact size/hash verification, canonical path checks, in-place native-update adoption, atomic state publication, retained signed releases, and rollback without lowering the highest observed sequence.

Installation records the raw user PATH registry value and type before adding one owned shim directory. Uninstall either restores that snapshot exactly or removes only the owned entry when the user changed PATH afterward. Cleanup is constrained to `%LOCALAPPDATA%\CLIEditor`; state load/save binds owned paths to that root, and installation, activation, and cleanup reject reparse-point redirects before file or PATH mutation.

The release design pins the upstream commit, Rust toolchain, GitHub Actions, patch hash, dependency locks, and license generator. Two independent MSVC builds must have identical complete artifact inventories. Both compile without the signing seed; only a separate post-parity protected job can sign the verified manifest. A separate Windows runner must install the downloaded ZIP, exercise both CLI routes and default/restore behavior, enforce the 50 ms p95 dispatcher-overhead budget for Codex and Claude, uninstall, and prove exact raw-PATH restoration before a draft release can publish.

## Evidence

- User live acceptance passed touchpad/wheel scrolling, text and image paste, click placement, drag selection, cursor behavior, and Ctrl+X/A/C/V in VS Code.
- CLI Editor has 66 passing unit/regression tests with warnings-denied clippy and formatting checks.
- The clean pinned Codex TUI aggregate has 3,557 passes, 26 known Windows snapshot failures, and 10 ignored tests.
- The patched aggregate has 3,570 passes, the exact same 26 failure names, and 10 ignored tests: 13 additional passing tests and no new failure.
- The reproducible patch is 35 files and 94,244 bytes; the publishable repository candidate remains well inside the deliberate 640 KiB lean-source budget rather than carrying a bundled upstream build tree.
- Third-party license reports render successfully for both Rust workspaces.

See `VERIFICATION_ai.md` for the detailed evidence boundary. GitHub-hosted MSVC execution, the draft release, downloaded-artifact acceptance, signing-key provisioning, and the final independent Claude verdict remain explicit gates until they actually pass.

## Engineering capabilities demonstrated

This project demonstrates terminal event-system design, Windows process and registry integration, supply-chain controls, signed update protocols, transactional recovery, compatibility policy, reproducible release automation, third-party licensing, adversarial review reconciliation, and evidence-based release management.
