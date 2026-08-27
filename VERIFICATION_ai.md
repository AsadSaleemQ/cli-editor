# Verification record

Date: 2026-08-26

## Result so far

The desktop-composer behavior is validated for pinned Codex 0.148.0 source and the CLI Editor dispatcher. General availability still requires a successful GitHub MSVC build, draft release, downloaded-artifact verification, and isolated Windows lifecycle test.

## User-reported live acceptance

In a real VS Code terminal using the earlier enhanced Codex build, the user confirmed laptop touchpad/wheel chat scrolling, text copy/paste, image paste, left-click placement and drag selection, mouse cursor behavior, and Ctrl+X/A/C/V. The user subsequently found that completed assistant text could not be selected immediately after typing unless a wheel event first released terminal mouse capture. The source fix below is automated-test validated but still awaits a fresh user live retest. These are user-reported live results and an explicit pending boundary, not automated end-to-end claims.

## Pinned Codex patch

- Upstream tag: `rust-v0.148.0`.
- Upstream commit: `3ba0f711642a888aec92a611a3f3b2211157ff89`.
- Patch: 35 files, 94,244 bytes.
- Patch SHA-256: `6a87f803bd7f6175c47419ecb0f254c0f62063239ce6007a01c670b259cff08c`.
- Clean pinned worktree `git apply --check`: passed.
- Patched worktree `git apply --reverse --check`: passed.
- `git diff --check`: passed.

The full clean pinned Codex TUI aggregate produced 3,557 passes, 26 known Windows snapshot failures, and 10 ignored tests. The full patched aggregate produced 3,570 passes, the exact same 26 failure names, and 10 ignored tests. The patch therefore adds 13 passing tests and no new failure or ignored test. The committed gate asserts all three aggregate counts and the exact failure-name set.

Focused regressions for wheel capture on composer and non-composer screens, turn-completion capture release with an empty composer, startup draft editing, and control-key preservation passed. The new `completed_turn_releases_capture_only_for_an_empty_composer` truth-table test proves completed transcript selection is handed to VS Code only when no draft would lose composer mouse editing. After the round-seven single-source mouse-disposition refactor, direct `rustfmt --check` passed, both affected capture regressions passed, and the complete patched aggregate gate reran successfully at 3,570/26/10 with the exact known failure-name set. An initial highly parallel local rebuild exhausted memory; the deterministic `-j 1` retry compiled and tested successfully. Earlier, `just fix -p codex-tui` passed with the pinned gnullvm toolchain. Repository-wide `just fmt` ran Rust formatting but could not run unrelated Bazel/Python formatters because this machine lacks dotslash/buildifier and its configured global uv cache is inaccessible. The direct Rust `cargo fmt --all -- --check` fallback passed.

An earlier exact pinned offline `codex-cli` release build passed after 156m57s. Raw optimized output was 1,274,960,896 bytes because of debug/line-table data; `llvm-strip --strip-all` reduced it to 226,393,088 bytes, and the stripped executable reported `codex-cli 0.148.0`. This local gnullvm artifact is evidence only; public packaging uses MSVC.

## CLI Editor dispatcher

- `cargo fmt --all -- --check`: passed.
- `cargo test --locked -j 1 --offline`: 66 passed, 0 failed.
- `cargo test --release --locked -j 1 --offline`: 66 passed, 0 failed with optimized code and production-only guards enabled.
- `cargo test --locked --offline -q -- --test-threads N`: 66 passed, 0 failed independently with `N=1`, `N=2`, and `N=8`, covering serial and concurrent harness scheduling.
- Release-sequence preflight: the entire release workflow uses one non-cancelling concurrency group, so dispatches cannot race the sequence check. The `prepare` job alone receives `contents: write`, allowing its release enumeration to include drafts while build and preflight jobs remain read-only. Prior manifests are downloaded by release tag into dedicated runner-temp files, with stderr kept out of JSON. The exact preparation block passed PowerShell parsing and mocked empty-history with harmless stderr, greater-sequence, duplicate-sequence rejection, enumeration-failure rejection, and download-failure rejection scenarios; Actionlint 1.7.12 passed after the workflow changes. A repeated sequence against a real draft remains a first-release hosted gate.
- Public command smoke: plain `cargo run -- --version` selects `cli-editor`, reports version 0.1.0 plus the unofficial/non-affiliation marker, and `--help` exposes all 10 documented management commands.
- Publication text hygiene: all 55 candidate files decode as strict UTF-8 without BOM, all nonempty files end with a newline, and every repository-relative Markdown link resolves. A temporary CRLF rewrite of the current review still matched its verdict and was detected as the normalized duplicate of its archived LF copy.
- Release-key negative acceptance: an isolated optimized build with the known development public key exited 125 before bundle reads or mutation and reported that release builds cannot embed the development signing key.
- Signed install dry run: an isolated debug-only development-key bundle passed detached-signature, exact size/SHA-256, dispatcher-minimum, enhanced-version probe, and native discovery checks; it rendered the install plan, safely left an unrelated Codex launcher unmanaged, discovered native Claude, exited 0, and left the raw user PATH unchanged.
- Toolchain metadata: `cargo metadata` reports Rust 1.95 as the declared minimum, matching the pinned 1.95 format, Clippy, test, build, and release jobs; no untested lower MSRV is advertised.
- `cargo clippy --locked --all-targets --offline -- -D warnings`: passed with Rust 1.95 gnullvm.
- After the round-eight rollback metadata fix, `cargo fmt --all -- --check`, warnings-denied Clippy, and all 66 debug launcher tests passed again; the signed update/rollback regression now checks the rollback digest and size against the retained signed manifest.
- After the round-nine fixes, formatting and warnings-denied Clippy passed; all 66 tests passed in debug and optimized release modes and independently at debug thread counts 1, 2, and 8. Focused Windows coverage removes a normal file with POSIX semantics and runs a copied test executable while proving the locked-image fallback removes its original command path. The hosted lifecycle now performs final uninstall through the installed shim and checks state, active shim names, bounded running-image residue, exact PATH restoration, and final owned-root cleanup.
- After the round-ten cleanup, all 66 tests passed offline in debug, optimized release, and debug thread-count 1/2/8 runs. Missing-native and installed-shim install regressions passed, and frozen cargo-about rendered both dependency-license reports without network access.
- After the round-eleven fixes, formatting, warnings-denied Clippy, and all 66 tests passed again in offline debug, optimized release, and independent debug thread-count 1/2/8 runs. The new extracted-bundle update-notice regression passed, the real Windows running-image deletion regression remained green, and the exact release-preparation block kept harmless `gh api` stderr out of successful JSON. Frozen dependency-license renders remained 114,925 bytes for the dispatcher and 1,138,566 bytes for pinned Codex. PowerShell parsing, Actionlint 1.7.12, and the isolated deterministic finalizer fixture passed. The publication scanner stopped only on the intentionally duplicated in-flight round-eleven report.
- After round-twelve approval, the two reviewer-prioritized pre-draft lows were cleared. An empty Cargo home performed the new locked dispatcher fetch, downloaded the full resolve graph including dev-dependencies, and then generated the frozen 114,925-byte dispatcher license report successfully. The rolling review-window wording no longer embeds stale round numbers.
- `cli-editor --version`: passed and identifies v0.1.0 as an unofficial distribution not affiliated with OpenAI, Anthropic, or Microsoft.
- PowerShell scripts parse: passed.
- Both workflow YAML files parse: passed.
- Actionlint 1.7.12 was downloaded from its GitHub release, verified against the published Windows SHA-256, and passed both workflows. CI independently downloads the pinned Linux archive, checks a hard-coded SHA-256, and runs the same semantic lint. Superseded CI runs on the same pull request/ref cancel automatically, while release runs serialize without cancellation.
- All six GitHub Actions dependencies are pinned to full commit SHAs; each SHA was resolved live through the GitHub API and matched its documented version tag on 2026-08-25.
- The 55-file candidate passes the size/privacy/supply-chain publication scanner and remains below the deliberate 640 KiB lean-source budget. A fresh exact-tree Claude report must still replace the prior current-review artifact before publication; this is a process/evidence gate even though the scanner cannot infer whether source changed after a verdict. The repeatable local and CI gate is in `scripts/check_publication_candidate_ai.ps1`: no credential patterns, private-key headers, email addresses, absolute local paths, binary files, reparse points, files over 1 MiB, duplicate current review, or oversized candidate.

Regression coverage includes 60-second cold native/release probe budgets plus identity-validated native fallback after an in-place update probe timeout, bounded 64 KiB temporary-file output capture that cannot hang on inherited descendant pipe handles, argument fidelity and literal delimiter escape, exact official npm resolution for both Codex and Claude, rejection of arbitrary script launchers, native child exit preservation and suspended-child cleanup on launcher setup failure, recursion safety, exact Codex versus warning-only host drift, explicit Claude warn-and-forward on unavailable or unlisted validation, opt-in strict managed Claude rejection, manifest expiry grace, enhanced metadata fast-path validation, state locking and transactional backup, root-bound owned-state validation with reparse-point rejection, artifact tamper detection, native target identity/adoption, staged update rollback, bounded audit history exposed through status/doctor JSON, PATH preservation, owned-root cleanup, and active-release comparison.

## Release and licensing validation

An isolated initial-commit smoke test confirmed that `git archive HEAD` contains exactly the 55-file publication candidate and no ignored workspace material. The release workflow validates dispatch inputs without shell interpolation, requires the requested version to equal `Cargo.toml`, and performs dispatcher preflight, pinned patch/lock validation, the full clean and patched TUI count/name gates, an MSVC `cargo check` of `codex-code-mode-host`, two independent build jobs sharing the prepared `SOURCE_DATE_EPOCH`, complete artifact parity, generated dependency-license reports, an isolated Windows install/default/explicit-route/restore/native-route/uninstall acceptance job with a 30-sample p95 native-versus-wrapper launch gate and raw PATH comparison, SBOM/provenance attestations, and draft-only publication. The pinned upload action's hidden-path behavior was verified from its exact README; every controlled `.artifacts` upload opts in explicitly, and the signed artifact excludes internal release tools. Exact unsigned and signed bundle/top-level allowlists reject undeclared files, and the finalizer rejects duplicate or missing manifest artifact names. The isolated finalizer fixture passed locally with the public development test key and is repeated in CI; it validates that the builder serializes the manifest before hashing it for the SBOM, binds signing to the prepared version/sequence/timestamps, rejects duplicate artifact names, and exercises schema/artifact checks, signing, deterministic ZIP creation, and checksum output without the production seed. Both build jobs run without the signing seed. The signer zeroizes its seed string, decoded bytes, stack seed, and signing-key material. Only the post-parity `sign-release` job receives it; that job performs manifest/artifact revalidation, deterministic signing/finalization, and no upstream compilation.

A local gnullvm `cargo check -p codex-code-mode-host` reached the Rusty V8 build boundary, where it could not download a gnullvm archive that upstream does not publish. This is not recorded as a successful host build; the hosted MSVC check/build/smoke test is the authoritative gate.

The release builder now asserts the pinned upstream MSVC stack/static-CRT configuration before patching, supplies those flags together with deterministic remap and `/Brepro` arguments, and inspects shipped PE imports and stack headers after stripping. Script parsing passed locally. Actual MSVC linkage/header execution remains part of the GitHub Windows build gate and is not claimed from the local gnullvm toolchain.

A local pinned `cargo-about` 0.9.1 render passed for both workspaces. The dispatcher report was 114,925 bytes and the Codex report was 1,138,566 bytes; both contained license text. The build asserts both generated reports are present inside the release ZIP.

## Release gates still pending

Before publishing a non-draft release:

1. obtain a fresh explicit Claude implementation verdict on the current tree;
2. receive explicit authorization to rotate/provision the release signing key, then configure the protected GitHub `release` environment secret without exposing it;
3. create the public `AsadSaleemQ/cli-editor` repository and run CI on GitHub's Windows MSVC runner;
4. build and inspect the draft release, attestations, SBOM, hashes, signature, source archive, ZIP, and generated license reports;
5. download the GitHub artifact and verify dry-run installation; and
6. exercise install/default/restore/update/rollback/uninstall on an isolated Windows user or clean machine, including PATH-collateral checks.

No Claude approval or hosted-release success is claimed in this record yet.
