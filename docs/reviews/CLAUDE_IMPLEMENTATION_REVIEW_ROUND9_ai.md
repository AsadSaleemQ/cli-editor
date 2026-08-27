I've completed the review. Here is the report.

---

▸ cache: none | claude-opus-5 | Skills: none | Context: fresh

# CLI Editor — independent implementation review (round 9)

Scope: `src/`, `src/bin/`, `patches/codex/rust-v0.148.0/`, `.github/workflows/*.yml`, `scripts/*.ps1`, `compatibility/`, `Cargo.toml`, `.gitignore`, and the public docs, verified against `CODEX_IMPLEMENTATION_RECONCILIATION_ai.md`. Read-only; no file was created, modified, moved, or deleted, and no ignored secret was read. `CLAUDE_IMPLEMENTATION_REVIEW_ai.md` being byte-identical to `docs/reviews/CLAUDE_IMPLEMENTATION_REVIEW_ROUND8_ai.md` is treated as the expected in-flight state.

## Reconciliation verification

Every round-8 resolution holds, and no earlier-round fix has regressed:

- **R8 L-1** — `CASE_STUDY_ai.md:32` now reports 91,626 bytes, matching `patches/codex/rust-v0.148.0/upstream.json`, `PATCH_REVIEW_ai.md:30`, `VERIFICATION_ai.md:17`, and `CODEX_IMPLEMENTATION_RECONCILIATION_ai.md:108`.
- **R8 L-2** — `src/installer.rs:743-751` takes `codex_sha256`/`codex_file_size` from `verified.manifest.artifact("codex-enhanced.exe")` and keeps only `codex_modified_unix_ms` from the file, matching install (`:164-176`) and update (`:418-429`). The regression asserts both rollback fields against the retained manifest (`src/installer.rs:1757-1763`).
- **R8 L-3** — `.github/workflows/release_ai.yml:64-71` downloads each prior manifest with `gh release download <tag> --repo … --pattern compatibility-manifest.json --output <RUNNER_TEMP file>`, keeps stderr in a separate file, and fails closed with the asset id plus the CLI error.
- **R7/R6/R5** — `prepare` still holds the job-level `contents: write` needed for draft enumeration (`release_ai.yml:26-27`) while `preflight`/`build-*`/`sign-release` inherit `contents: read` (`:21-22`); the publication gate's verdict regex and duplicate comparison are CRLF-tolerant (`scripts/check_publication_candidate_ai.ps1:43-48`); both probe budgets are 60 s (`src/discovery.rs:330-333`) with the identity-validated fallback at `src/dispatcher.rs:83-107`; `verify_managed_codex_compatibility` (`src/installer.rs:453-465`) returns `Ok(())` with no managed Codex; install re-verifies all three copied executables inside the transaction (`:117-119`); the console-teardown counter pattern in `src/process.rs:111-139` is intact.

**Patch provenance (measured):** 91,626 bytes, SHA-256 `a4fab6ee7c59c305fb0cc5b878cae0fa0fad44e053b1670edb13eda1c93b93a3`, 34 `diff --git` entries, zero CR bytes — matching `upstream.json`, `scripts/build_release_ai.ps1:17`, and `.github/workflows/ci_ai.yml:71`. `prepare_mouse_capture(event, respect_alt_screen)` is single-sourced (patch lines 2012-2058), `prepare_mouse_capture_for_non_composer_event` is invoked from the eight non-composer loops, and `_ => None` is still absent from `event_stream.rs` (patch line 2095), so a new upstream `Event` variant fails compilation.

**Item 6 — the count-and-name gate is honest.** `scripts/check_codex_tui_baseline_ai.ps1:41-51` aggregates every `test result:` summary and asserts passed, failed (26) and ignored (10); only `ExpectedPassed` is parameterized. It then requires exact set equality with the baseline file, which I measured at 26 lines and 26 unique names, then requires a nonzero cargo exit (`:63`) before resetting `$LASTEXITCODE`. Both workflows drive it at 3557 (clean) and 3569 (patched). `VERIFICATION_ai.md:23` states the delta as "+12 passing, no new failure or ignored test" without overclaiming. Dispatcher `#[test]` count measured: 58, matching the claim.

**Publication gate (executed read-only):** exactly one finding — `duplicate-current-review` on `CLAUDE_IMPLEMENTATION_REVIEW_ai.md`, the expected in-flight state. Candidate measured at 56 files, 505,720 bytes.

**Signing handoff:** `build_release_ai.ps1:155` serializes the manifest before `:173` hashes it, statically enforced by `check_release_finalizer_ai.ps1:7-11`; `finalize_release_ai.ps1:64-93` schema-validates, revalidates version/issue/sequence/expiry, requires the exact three-name artifact inventory with matching size and digest, then refuses to proceed unless the signer's derived public key equals `compatibility/public-key.hex`. `production_key_verifies_the_committed_release_fixture` and `configured_release_key_is_valid_and_not_the_development_key` (`src/compatibility.rs:186-209`) hold, alongside the runtime guard at `src/installer.rs:815-817`.

## Findings

### High

**H-1 — `cli-editor uninstall` run through the installed shim — the documented removal path — aborts fatally after it has already mutated PATH, and no CI job covers it.**
`src/installer.rs:1090-1094` (`remove_or_defer(&shims.join(name))?`) with `src/installer.rs:1202-1226` (`remove_or_defer` → `defer_delete`).

`install` copies the dispatcher to `<owned root>\shims\cli-editor.exe` (`src/installer.rs:130-133`) and prepends that directory to the user PATH (`:143-149`). `README.md:13-20` then tells the user to open a new terminal and run `cli-editor uninstall`, so the resolved command *is* the shim copy, and that process is uninstalling its own running image.

Failure scenario: inside `store.remove_with`, PATH is restored first (`src/installer.rs:1076-1081`), then the loop reaches `shims\cli-editor.exe`. Windows cannot delete a running executable image, so `std::fs::remove_file` returns ERROR_ACCESS_DENIED (5) or ERROR_SHARING_VIOLATION (32), which routes to `defer_delete`. `defer_delete` calls `MoveFileExW(path, NULL, MOVEFILE_DELAY_UNTIL_REBOOT)`, which Microsoft documents as usable **only** by a process running as an administrator or LocalSystem — CLI Editor is a deliberately per-user, non-elevated install, so this fails and returns `Err`. The `?` aborts `remove_with` *before* `state.json` and `state.backup.json` are deleted (`src/state.rs:276-282`) and before `cleanup_owned_root` runs (`src/installer.rs:1097`). Net result: exit 125, user PATH already reverted, the entire `%LOCALAPPDATA%\CLIEditor` tree still present, and every retry reproduces the same failure. A subsequent `cli-editor install` then reports "already installed; existing state was revalidated and republished" (`:195-197`) without repairing anything, so the user has no in-product recovery.

This contradicts `UPDATE_AND_ROLLBACK_ai.md:40` ("In-use owned files are scheduled for deletion at restart and any residue is reported") and the single-command-removal goal. It is also invisible to automation: `.github/workflows/release_ai.yml:312` and `:315` invoke uninstall through `$editor.FullName`, the copy extracted from the ZIP, never through `shims\cli-editor.exe`, so `lifecycle-acceptance` passes while the documented flow fails.

Recommended correction, in three parts:
1. Make shim removal non-fatal in the `remove_with` closure — collect failures and warn, exactly as `cleanup_owned_root` already does at `src/installer.rs:1147-1154`, which re-attempts the same files. State removal and root cleanup then complete, and the worst case is one orphaned executable plus a printed notice.
2. Harden `defer_delete`: renaming a running image *is* permitted on Windows, so first `MoveFileExW(path, <owned-root>\.pending-delete.<pid>.<n>, MOVEFILE_REPLACE_EXISTING)` (or into `%TEMP%`) so the shim directory stops intercepting `codex`/`claude`/`cli-editor` immediately, and only then attempt the reboot queue on a best-effort basis.
3. Add a hosted acceptance step that performs the final uninstall through `<shims>\cli-editor.exe` rather than the extracted copy, so this path is covered before publication.

Note for (1): after the fix, a shell that still holds the pre-uninstall PATH would get `NotInstalled` from a leftover `codex.exe`/`claude.exe` shim until a new terminal is opened. That is a strictly better outcome than the current hard failure, but it is worth a sentence in `UPDATE_AND_ROLLBACK_ai.md`.

### Low

**L-1 — Two `codex` launches racing the first post-update adoption make one of them fail outright instead of falling back.**
`src/dispatcher.rs:83-102` and `src/installer.rs:1021-1034`.

After `npm install -g @openai/codex@<next>`, the first launch sees `TargetChanged` and calls `adopt_in_place`, which opens a state transaction and rejects the write with `StateChangedDuringOperation` if another process already adopted (`src/installer.rs:1027-1032`). In `revalidate_native_target_with`, only `VersionProbeTimedOut` has a fallback arm; `StateChangedDuringOperation` hits `Err(error) => Err(error)` and propagates out of `run_managed`, so the second concurrent `codex` invocation exits 125 with "state changed while the operation was being prepared; retry". Failure scenario: a terminal profile, task runner, or split pane starts two Codex sessions within the same second after a routine CLI update — one session simply dies. Recommended correction: on `StateChangedDuringOperation`, reload state once and re-run `validate_native_metadata`; if the freshly adopted record now validates, continue with it instead of returning an error.

**L-2 — `repair --adopt-native` installs the *rolled-back* release's dispatcher as a shim, mixing dispatcher versions in the shim directory.**
`src/installer.rs:962-967` and `:982`.

`repair` copies `prepared.active_release.directory.join("cli-editor.exe")` into the shim directory, but `rollback` deliberately leaves the newest dispatcher shims in place while pointing `active_release` at an older retained release (`UPDATE_AND_ROLLBACK_ai.md:32`, `src/installer.rs:634-641`). Failure scenario: a user updates to dispatcher 0.2, rolls back the payload, then installs Claude and runs `cli-editor repair --adopt-native claude`; `claude.exe` becomes a 0.1 dispatcher while `cli-editor.exe` and `codex.exe` remain 0.2. If a future release advances `STATE_SCHEMA_VERSION`, the stale shim fails `UnsupportedStateSchema` on every `claude` invocation. This cannot occur at v0.1.0 (only one dispatcher version exists), so it is a latent defect rather than a live one. Recommended correction: source the repair shim from `state.shim_directory.join("cli-editor.exe")`, which is by construction the newest dispatcher.

**L-3 — The publication candidate sits at 96.5 % of its own size gate, with the only adjustable document being the current review.**
`scripts/check_publication_candidate_ai.ps1:2` (`$MaximumTotalBytes = 524288`).

Measured: 56 files, 505,720 bytes — 18,568 bytes of headroom, of which `CLAUDE_IMPLEMENTATION_REVIEW_ai.md` currently occupies 14,760. Failure scenario: the next current review is ~33 KB or the round-9 archive is added under `docs/reviews/`, and `ci_ai.yml:23` starts failing every push to `main` with `oversized-candidate` — a gate failure caused by review bookkeeping rather than by anything about the shipped product. Recommended correction: move `CODEX_IMPLEMENTATION_RECONCILIATION_ai.md` (21,806 bytes) under `docs/`, or raise `$MaximumTotalBytes` and record the new intent, before the next round is archived.

## Residual validation gaps (not defects; release gates)

1. **Signing seed.** `compatibility/public-key.hex` validates its production fixture and `scripts/finalize_release_ai.ps1:91-93` fails closed on any other key, so no release can be produced until the seed is provisioned or the key rotated. Correctly escalated as an external authorization gate; no ignored secret was read.
2. **Official Claude npm layout.** `src/discovery.rs:320-323` hard-requires `<package_root>\bin\claude.exe`. Only `release_ai.yml:227-229` can settle whether the pinned Windows layout matches; if it does not, `install` leaves Claude unmanaged with a warning and `default all --strict` at `:296` fails visibly, blocking the draft rather than shipping a broken route.
3. **Bit-for-bit parity of the patched upstream build.** `verify-reproducibility` (`release_ai.yml:320-346`) is the only thing proving the two independent MSVC builds agree. `SOURCE_DATE_EPOCH`, `/Brepro`, and remapped prefixes are all wired, but `build_release_ai.ps1:65-70` sets `CARGO_ENCODED_RUSTFLAGS`, which overrides any `build.rustflags` in the upstream `codex-rs` cargo config, and no local evidence covers an upstream build embedding a timestamp or VCS stamp. Failure here blocks the release rather than shipping a bad artifact, but it is a plausible cause of the first dispatch failing.
4. **`codex-code-mode-host` MSVC build and `--help` exit code.** The hosted `cargo check` (`ci_ai.yml:101`, `release_ai.yml:126-128`) plus the builder smoke probe (`build_release_ai.ps1:129-130`) is authoritative and unrun; the local GNU/LLVM attempt stops at the Rusty V8 boundary. The probe assumes `--help` exits 0.
5. **cargo-about report parity.** `about_ai.hbs` carries no timestamp, so parity is expected but proven only by `verify-reproducibility`.
6. **Repository has no commits yet** (`git log` reports an unborn `main`), so `git archive HEAD` (`build_release_ai.ps1:198`) and `git ls-files --cached` in the publication gate are exercised only against the 56-file untracked candidate. Re-run both after the initial commit.
7. **Repeated-sequence rejection against a real draft, GitHub Actions execution, draft-release inspection, and clean-machine install acceptance** remain untried by construction.

Nothing in the Rust, the patch, the workflows, or the scripts is structurally unable to produce a first successful release once the seed exists: input validation before secret-bearing jobs, parity-before-signing, exact unsigned and signed allowlists, deterministic ZIP, attestations, and draft-only publication are wired end to end, and the CI finalizer fixture exercises the build→sign handoff with a development key.

H-1 is the one item blocking approval. It is a shipped-code defect on the primary removal path, it leaves the user in a partially-uninstalled state with no in-product recovery, and it is specifically not covered by the hosted acceptance job — so publishing and running draft-release CI would not surface it. The fix is contained to `src/installer.rs` plus one acceptance step. L-1 through L-3 do not need to land first.

**Files changed:** none. This review was entirely read-only.

VERDICT: CHANGES_REQUIRED

◂ used: ~6.9k | saved: none | Skills: none | Context: fresh
