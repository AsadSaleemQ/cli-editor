I've completed the review. Here is the report.

---

▸ cache: none | claude-opus-5 | Skills: none | Context: fresh

# CLI Editor — independent implementation review (round 11)

Scope: `src/`, `src/bin/`, `patches/codex/rust-v0.148.0/`, `.github/workflows/*.yml`, `scripts/*.ps1`, `compatibility/`, `Cargo.toml`, `.gitignore`, and the public docs, verified against `CODEX_IMPLEMENTATION_RECONCILIATION_ai.md`. Read-only; no file was created, modified, moved, or deleted, and no ignored secret was read. `CLAUDE_IMPLEMENTATION_REVIEW_ai.md` being byte-identical to `docs/reviews/CLAUDE_IMPLEMENTATION_REVIEW_ROUND10_ai.md` is treated as the expected in-flight state.

## Reconciliation verification

Every round-10 resolution holds except one that was explicitly escalated, and no round-9 or earlier fix has regressed.

- **R10 L-1** — fixed. `CliEditorError::NativeTargetMissing` exists (`src/error.rs:17-20`), is in the 126 class (`src/error.rs:118`), and is produced from both validation paths (`src/dispatcher.rs:356-366`, `src/discovery.rs:130-140`). `src/doctor.rs:113-119` surfaces the same actionable detail. Regressions at `src/dispatcher.rs:582-603` and `src/discovery_tests.rs:17-39`.
- **R10 L-2** — fixed. `src/installer.rs:31-38` short-circuits before bundle discovery; `is_installed_cli_editor_shim` (`:211-223`) compares the canonical parent against the canonical shim directory, with the external-executable case covered at `:1479-1495`.
- **R10 L-3** — fixed. `scripts/build_release_ai.ps1:97` passes `--frozen`. Corroborated: both frozen renders exist at exactly the claimed 114,925 / 1,138,566 bytes.
- **R10 L-4** — deliberately not done and escalated. See "Escalations" below; not treated as a defect.
- **R10 L-5** — fixed. `RELEASE_NOTES_ai.md:9` now carries the owned/unchanged qualification and the "preserving later edits" clause.
- **R10 L-6/L-7** — fixed. `scripts/check_publication_candidate_ai.ps1:2` is a deliberate 655,360-byte budget; I measured the candidate at **55 files, 523,547 bytes** (131,813 bytes of headroom), matching `VERIFICATION_ai.md:37,50`.
- **R9 H-1** — still fixed. `remove_owned_shims_with` (`src/installer.rs:1121-1131`) cannot abort `remove_with`, `defer_delete` (`:1244-1302`) tries POSIX-semantics deletion first and then `MoveFileExW(..., MOVEFILE_REPLACE_EXISTING)`, and `release_ai.yml:312-326` drives the final uninstall through `<shims>\cli-editor.exe`.
- **Rounds 4-8** — `prepare` alone holds `contents: write` (`release_ai.yml:26-27`); the verdict regex and duplicate comparison are CRLF-tolerant; both probe budgets are 60 s; install/update/rollback all record the manifest-declared digest and size; the console-teardown counter pattern (`src/process.rs:111-139`) is intact.

**Patch provenance (measured):** 91,626 bytes, SHA-256 `a4fab6ee7c59c305fb0cc5b878cae0fa0fad44e053b1670edb13eda1c93b93a3`, 34 `diff --git` entries resolving to 34 unique paths, all under `codex-rs/tui/src/**`, zero CR bytes. Matches `upstream.json`, `build_release_ai.ps1:17`, `ci_ai.yml:71`, `PATCH_REVIEW_ai.md:30`, `VERIFICATION_ai.md:17`, and `CASE_STUDY_ai.md:32`. `prepare_mouse_capture(event, respect_alt_screen)` is single-sourced (patch 2026-2052), `_ => None` is absent from `event_stream.rs` (patch 2095), and every prompt/picker loop consumes or routes `TuiEvent::Mouse`.

I also independently checked the two highest-risk new code paths in the patch against pinned upstream source. `byte_pos_at_cell` (patch 833-868) uses `line.end.saturating_sub(1)`, which is the *same* sentinel-byte convention as upstream `bottom_pane/textarea/wrapping.rs:61,109,118,136` — every emitted range end is `<char-boundary>+1`, so the slice at patch line 854 cannot land mid-UTF-8. The selection overlay (patch 1049-1064) reuses the already-sentinel-stripped `line_range` (`textarea.rs:2036`) and mirrors the existing element/highlight overlays exactly. No panic path found.

**Item 6 — the committed gate represents the evidence honestly, with one caveat.** `scripts/check_codex_tui_baseline_ai.ps1:41-51` aggregates *every* `test result:` summary and asserts passed, failed (hard-coded 26) and ignored (hard-coded 10); only `ExpectedPassed` is parameterized. It requires exact set equality with `compatibility/codex-tui-windows-baseline-failures_ai.txt` (measured: 26 lines, 26 unique names), then requires a nonzero cargo exit (`:63`) before resetting `$LASTEXITCODE`. Both workflows drive it at 3557 (clean) and 3569 (patched). `VERIFICATION_ai.md:23` states the delta as "+12 passing, no new failure or ignored test" without overclaiming. Launcher `#[test]` count measured: **65**, matching every claim. The caveat is H-1: the gate compiles the upstream tree under upstream's own cargo config, while the release job compiles it with those flags replaced, so the tested and shipped configurations differ.

**Publication gate (executed read-only):** exactly one finding — `duplicate-current-review`, the expected in-flight state.

## Findings

### High

**H-1 — The release builder replaces upstream Codex's pinned MSVC rustflags instead of extending them, so the shipped `codex-enhanced.exe` and `codex-code-mode-host.exe` are built without `+crt-static` and without the 8 MiB main-thread stack that upstream deliberately requires. No gate detects this.**
`scripts/build_release_ai.ps1:65-70`.

The builder sets

```
$env:CARGO_ENCODED_RUSTFLAGS = @(
    "--remap-path-prefix=$root=Z:/cli-editor",
    "--remap-path-prefix=$upstream=Z:/codex",
    '-C', 'link-arg=/Brepro'
) -join [char]0x1f
```

before `cargo build ... -p codex-cli -p codex-code-mode-host` at `:73`. Pinned upstream ships `codex-rs/.cargo/config.toml` containing:

```
[target.'cfg(all(windows, target_env = "msvc"))']
rustflags = ["-C", "link-arg=/STACK:8388608", "-C", "target-feature=+crt-static"]
```

Cargo documents these as **four mutually exclusive sources**, checked in order: `CARGO_ENCODED_RUSTFLAGS`, then `RUSTFLAGS`, then matching `target.<triple>`/`target.<cfg>` config entries, then `build.rustflags`. Setting the environment variable therefore discards the config entry entirely rather than merging with it. Nothing in the repository re-adds either flag — I searched the whole publication candidate for `crt-static`, `/STACK`, `RUSTFLAGS`, and `link-arg`; the only hits are `build_release_ai.ps1:65,69`, the round-9 archive, and the reconciliation row itself.

Two concrete failure scenarios:

1. **Stack.** `/STACK:8388608` raises the PE reserve for the main thread, which is where `#[tokio::main]`'s `block_on` runs the TUI render/event loop. Without it the shipped enhanced Codex gets the 1 MiB MSVC default. That this codebase needs deep stacks is not speculative — `scripts/check_codex_tui_baseline_ai.ps1:9` sets `RUST_MIN_STACK = 16777216` to keep the TUI suite from overflowing, and upstream sets the same 8 MiB reserve for all three Windows targets. A user rendering a deeply nested markdown/diff cell gets `0xC00000FD` with no message — the exact "malfunction rather than fail safely" outcome the project sets out to avoid.
2. **CRT.** Without `+crt-static` the two Codex executables import `VCRUNTIME140.dll`, which ships with the Visual C++ Redistributable rather than with Windows. On a machine without it, `codex cli-editor` dies at process start with a loader error.

Neither is catchable by the current pipeline. `ci_ai.yml:102-106` and `release_ai.yml:114-129` run the TUI suite and the code-mode-host check **without** `CARGO_ENCODED_RUSTFLAGS`, i.e. under the correct flags; only the release build uses the wrong ones. The builder's smoke probes (`build_release_ai.ps1:121-130`) and `lifecycle-acceptance` both run on GitHub runners that have the redistributable installed and never stress the stack.

This also means `CODEX_IMPLEMENTATION_RECONCILIATION_ai.md:39` ("L8 build flags overwrote upstream flags | Build uses `CARGO_ENCODED_RUSTFLAGS` with separate remap and `/Brepro` arguments") records the encoding fix as the resolution while the overwrite it was named for is still present.

Recommended correction:
1. Set `CARGO_ENCODED_RUSTFLAGS` **per build** rather than once. For the upstream build, prepend upstream's exact MSVC flags: `-C␟link-arg=/STACK:8388608␟-C␟target-feature=+crt-static␟--remap-path-prefix=…␟-C␟link-arg=/Brepro`. Leave the dispatcher build with remap + `/Brepro` (plus M-1 below).
2. Add a preflight assertion that the pinned commit's `codex-rs/.cargo/config.toml` MSVC `rustflags` array still equals the list the builder hard-codes, so a future upstream bump that adds a flag fails the build instead of silently dropping it. This belongs next to the existing patch-SHA check at `build_release_ai.ps1:36`.
3. Add a post-link assertion that `codex-enhanced.exe` has no `VCRUNTIME140.dll` import (`llvm-readobj --coff-imports`, already available from `llvm-tools-preview`), placed with the other smoke probes at `:121-130`.

Note for sequencing: this changes every produced binary, so it must land **before** the first `verify-reproducibility` / draft-release dispatch, not after.

### Medium

**M-1 — `cli-editor.exe` itself is linked against the dynamic MSVC CRT, so the very first command in the documented install flow can fail on a clean Windows machine.**
`Cargo.toml` (no `[target.'cfg(windows)']` rustflags) with `scripts/build_release_ai.ps1:65-71`; the repository has no `.cargo/config.toml`.

`README.md:7-11` tells a user to download the ZIP and run `.\cli-editor.exe install`. Built for `x86_64-pc-windows-msvc` with the default dynamic CRT, that executable imports `VCRUNTIME140.dll`. The UCRT (`ucrtbase.dll`, `api-ms-win-crt-*`) is in-box on Windows 10+, but the VC++ runtime is not — it arrives with the Visual C++ Redistributable. On a machine that has never had it installed, the user gets a Windows loader dialog instead of the launcher, with no diagnostic and nothing in the product able to help. This is distinct from H-1 because there is no upstream config to preserve here — the flag was simply never set — and because it defeats single-command installation rather than runtime behavior.

Recommended correction: add `-C target-feature=+crt-static` to the dispatcher build's rustflags (all dependencies are pure Rust, so this is free), and extend the H-1 import assertion to `cli-editor.exe`. Keep the flag identical across `build-primary` and `build-reproducibility` so parity is unaffected.

### Low

**L-1 — `CASE_STUDY_ai.md:32` claims the candidate "remains under 0.5 MiB", a threshold that publishing this report will cross.**
Measured candidate: 523,547 bytes against 524,288, i.e. 741 bytes of margin, while the file this review replaces is 18,739 bytes. Any report longer than that falsifies the sentence on the same commit that publishes it. The real gate is the deliberate 640 KiB budget in `scripts/check_publication_candidate_ai.ps1:2`, so the prose is asserting a stricter number than anything enforces. Recommended correction: restate it as "well inside the deliberate 640 KiB lean-source budget" so the claim tracks the gate rather than an incidental measurement.

**L-2 — Self-uninstall prints two different residue paths, the first of which no longer exists by the time the user reads it.**
`src/installer.rs:1290-1300` with `:1179-1186`.

`remove_owned_shims` renames the running `cli-editor.exe` to `.pending-delete.<pid>.<nanos>.exe` and, when the non-elevated restart queue fails, prints "remove `<…>\.pending-delete.A.exe` after this command exits". `cleanup_owned_root` then enumerates the owned tree, finds that same file still locked, and renames it again to `.pending-delete.<pid>.<nanos2>.exe`, printing a second, different path. The user sees two instructions; following the first one fails because path A was consumed by the second rename. The acceptance assertion at `release_ai.yml:319-323` is unaffected (still exactly one residue file), so this is purely message quality. Recommended correction: have `remove_or_defer`/`defer_delete` collect the final residue path and print once, after `cleanup_owned_root` completes.

**L-3 — Release preparation folds `gh` stderr into the stream it parses as JSON, so a routine CLI notice aborts the release with an unrelated error.**
`.github/workflows/release_ai.yml:59-61`.

`$releaseOutput = @(& gh api "repos/…/releases?…" 2>&1)` merges stderr into the array that `:61` passes to `ConvertFrom-Json`. `gh` writes upgrade notices and deprecation warnings to stderr while still exiting 0, so `$LASTEXITCODE -ne 0` at `:60` does not catch it and the JSON parse fails with "Conversion from JSON failed" — a confusing stop with no indication that release enumeration was fine. The prior-manifest download at `:66` already models the right pattern (stderr to a separate file). It fails closed, which is why this is low. Recommended correction: redirect stderr to a `$env:RUNNER_TEMP` file the way `:66-70` does, and surface it only in the throw.

**L-4 — The publication credential scan does not recognize fine-grained GitHub tokens.**
`scripts/check_publication_candidate_ai.ps1:29`.

The pattern covers `gh[pousr]_…` (classic PATs), OpenAI `sk-…`, AWS `AKIA…`, and PEM private-key headers, but not GitHub's fine-grained format `github_pat_…`, which is now the default for new tokens and does not match `gh[pousr]_`. A fine-grained PAT pasted into a doc or workflow would pass the boundary check that exists specifically to stop that. Recommended correction: add `\bgithub_pat_[A-Za-z0-9_]{30,}\b` to the alternation.

**L-5 — Running `install` from a *newer* extracted bundle when CLI Editor is already installed reports success and changes nothing, without naming `update --bundle`.**
`src/installer.rs:102-105` and `:203-207`.

The transaction closure returns `Ok((existing, false))` whenever state already exists, after the new bundle has been fully signature- and hash-verified. The user sees "CLI Editor is already installed; existing state was revalidated and republished." and exit 0, with no hint that the newer release they just downloaded was not activated. `README_ai.md:24` documents `update --bundle`, but the message is the one they are actually looking at. Recommended correction: when the verified bundle's manifest sequence exceeds `state.highest_manifest_sequence`, print the exact `cli-editor update --bundle <source directory>` command instead of the generic already-installed line.

## Verified during this round (previously open gaps now closed)

- **Official Claude npm layout** (round 9 gap 2, round 10 gap 3) — closed by direct inspection of the pinned `@anthropic-ai/claude-code@2.1.240` tarball: it contains `package/bin/claude.exe` and declares `"bin": { "claude": "bin/claude.exe" }`. That is exactly what `src/discovery.rs:326` and `is_expected_npm_claude_path` (`:205-209`) require, reached through the `claude.cmd` shim npm generates. Hosted acceptance still confirms end to end, but the layout risk is gone.
- **`cargo about generate --frozen`** — the flag is accepted and both frozen reports exist at the byte sizes `VERIFICATION_ai.md:60` claims.
- **PowerShell 7 invoking `\\?\`-verbatim native paths** — confirmed, which is what `release_ai.yml:285-295` depends on when it reads `state.json` paths for the p95 latency probes.

## Residual validation gaps (not defects; release gates)

1. **Signing seed.** `compatibility/public-key.hex` validates its production fixture (`src/compatibility.rs:186-198`) and `scripts/finalize_release_ai.ps1:91-93` fails closed on any other key, so no release can be produced until the seed is provisioned or the key rotated. Correctly escalated; no ignored secret was read.
2. **Bit-for-bit parity of the patched upstream build.** Unrun, and its inputs change with the H-1 fix. `SOURCE_DATE_EPOCH`, `/Brepro`, and both remap prefixes are wired, and the remap ordering is correct (rustc applies the last matching mapping, so the nested `$upstream` prefix wins).
3. **Windows mouse-capture release/restore.** `windows_console::set_mouse_capture` is `#[cfg(all(windows, not(test)))]` (patch 2148-2161), so every automated test exercises only the boolean. `set_modes` arms capture through crossterm's DECSET path while release/restore toggle the console-input `ENABLE_MOUSE_INPUT` bit; under ConPTY those are not the same mechanism. `VERIFICATION_ai.md:11` correctly labels the wheel/scroll handoff as user-reported live acceptance rather than an automated claim. Re-confirm on the draft artifact in both `conhost` and Windows Terminal.
4. **`codex-code-mode-host` MSVC build and `--help` exit code.** The hosted `cargo check` (`ci_ai.yml:101`, `release_ai.yml:126-128`) plus the builder smoke probe (`build_release_ai.ps1:129-130`) is authoritative and unrun; the local GNU/LLVM attempt stops at the Rusty V8 boundary.
5. **Draft-release resolution by tag.** `release_ai.yml:66` reads each prior manifest with `gh release download <tag_name>`; during the draft-only v0.1 phase this relies on the CLI's draft fallback, which is unexercised. Fails closed.
6. **Repository has no commits yet**, so `git archive HEAD` (`build_release_ai.ps1:198`) and `git ls-files --cached` in the publication gate are exercised only against the 55-file untracked candidate. Re-run both after the initial commit.
7. **Production-signed `update`/`rollback` on a downloaded draft**, and **repeated-sequence rejection against a real draft**, remain untried by construction. `signed_update_and_rollback_preserve_sequence_and_previous_release` plus the development-key finalizer fixture (`ci_ai.yml:53`) bound the risk.
8. **Runner budget.** Each `patch`/`preflight` job compiles and runs the full `codex-tui` suite twice inside 360 minutes, and each build job rebuilds the workspace at `codegen-units=1` with thin LTO. Plausible, but unmeasured on hosted hardware.

## Escalations for the user (not defects, not reviewer deadlock)

- **Signing-key provisioning/rotation** and **creating the public `AsadSaleemQ/cli-editor` repository** are external authorization gates, already recorded at `VERIFICATION_ai.md:66-71`.
- **`SECURITY.md` / `CONTRIBUTING.md` at GitHub-recognized paths** (round-10 L-4). GitHub links a security policy only from `SECURITY.md` and a contributing guide only from `CONTRIBUTING.md`, in the root, `docs/`, or `.github/`. The workspace `_ai` naming rule forbids those exact names, and Codex correctly declined to create them unilaterally. This needs a one-line decision from you: either grant a named exception for these two platform-reserved filenames (as already exists in practice for `README.md`, `LICENSE`, and `NOTICE`), or accept that the Security tab will read "no security policy" at publication. Enabling private vulnerability reporting in repository settings is a separate publication gate either way.

## Assessment

The Rust is sound. Discovery, native self-adoption, manifest verification, enhanced/native selection, Windows process forwarding, state transactions, install/update/rollback compensation, PATH mutation, and self-uninstall all hold up, and the round-9 high finding is genuinely fixed in shipped code, in unit coverage, and in the hosted acceptance job. The patch is provenance-clean, exhaustiveness-guarded, and free of the byte-boundary hazards I went looking for. The workflows and scripts are structurally capable of producing a first successful release once the seed exists.

What blocks approval is H-1 with M-1: the release automation silently discards the link flags the pinned upstream requires, and it is precisely the automation, not the source, that decides what users run. The tested configuration and the shipped configuration diverge, so none of the strong evidence in `VERIFICATION_ai.md` characterizes the binary that would go into the draft — and the two most likely symptoms, a silent stack-overflow abort and a missing-DLL loader failure, are both invisible on GitHub runners. The fix is small and localized to `scripts/build_release_ai.ps1`, but it must land before the first parity/draft dispatch because it changes every artifact hash. L-1 through L-5 are independent hardening and message-quality items that can ship alongside it.

Codex should proceed with H-1, M-1, and L-1 through L-5 without further instruction, re-run the local dispatcher gates and the frozen license renders, refresh the reconciliation row for L8 to describe the actual preservation fix, and return the tree for a fresh verdict. The signing seed, the public repository, and the `SECURITY.md` naming decision are the only items that need you.

**Files changed:** none. This review was entirely read-only.

VERDICT: CHANGES_REQUIRED

◂ used: ~9.1k | saved: none | Skills: none | Context: fresh
