# Third-party notices

CLI Editor includes or builds upon third-party software under their respective licenses.

## OpenAI Codex

The enhanced Codex artifact is a modified build of `openai/codex`, pinned to tag `rust-v0.148.0` and commit `3ba0f711642a888aec92a611a3f3b2211157ff89`. Upstream is licensed under Apache License 2.0. The upstream `LICENSE` and `NOTICE` files are reproduced at the repository root and must accompany binary distributions.

CLI Editor's changes are represented by `patches/codex/rust-v0.148.0/0001-desktop-composer.patch`. The patch adds mouse capture/routing, touchpad and wheel scrolling, click placement, drag selection, clipboard editing, undo/redo, selection rendering, and related tests/snapshots. It also updates all affected interactive screens to handle the added mouse event.

## Rust dependencies

Rust dependency names, versions, sources, checksums, and license metadata are fixed by `Cargo.lock` files and generated build metadata. Release automation retains those lockfiles, emits a CycloneDX SBOM, and packages generated full dependency-license reports for both CLI Editor and the modified Codex binaries.

All product and company names identify compatibility only and remain trademarks of their respective owners.
