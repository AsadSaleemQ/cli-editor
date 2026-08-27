# CLI Editor: independent implementation review

Fresh exact-tree review of the current working tree. This report supersedes every earlier verdict and inherits no approval from them. Scope covered: `src/`, `src/bin/`, `patches/codex/rust-v0.148.0/`, `.github/workflows/*.yml`, `scripts/*.ps1`, `compatibility/`, `Cargo.toml`, `.gitignore`, and the public documents, checked against `CODEX_IMPLEMENTATION_RECONCILIATION_ai.md`, `DESKTOP_COMPOSER_BEHAVIOR_ai.md`, `PATCH_REVIEW_ai.md`, and `VERIFICATION_ai.md`.

Method: read-only inspection. No file was created, modified, moved, or deleted other than this report. No ignored signing secret was read. `.work/`, `.trash/`, `target/`, and the archived rounds under `docs/reviews/` were treated as context only, never as authority; the patch file itself and the committed sources are the authority for every claim below.

## Measured provenance

Independently recomputed against the current tree:

| Property | Measured | Records that agree |
|---|---|---|
| Patch bytes | 94,244 | `patches/codex/rust-v0.148.0/upstream.json:7`, `PATCH_REVIEW_ai.md:30`, `VERIFICATION_ai.md:17`, `CASE_STUDY_ai.md:32` |
| Patch SHA-256 | `6a87f803bd7f6175c47419ecb0f254c0f62063239ce6007a01c670b259cff08c` | `upstream.json:6`, `scripts/build_release_ai.ps1:17`, `.github/workflows/ci_ai.yml:71`, `PATCH_REVIEW_ai.md:30`, `VERIFICATION_ai.md:18` |
| `diff --git` entries | 35, resolving to 35 unique paths, all under `codex-rs/tui/src/**` | `upstream.json:8`, `PATCH_REVIEW_ai.md:30` |
| CR bytes in patch | 0 | consistent with the LF-only contract |
| Baseline failure list | 26 lines, 26 unique names | `compatibility/codex-tui-windows-baseline-failures_ai.txt` |
| Launcher `#[test]` and `#[tokio::test]` attributes | 66 across `src/**` | `VERIFICATION_ai.md:45`, `CODEX_IMPLEMENTATION_RECONCILIATION_ai.md:24` |
| Publication candidate | 55 files, 549,791 bytes against the 655,360-byte budget | `VERIFICATION_ai.md:37,52` |

No provenance record in the tree contradicts a measurement.

## Item 6: does the committed gate represent the evidence honestly

Yes. `scripts/check_codex_tui_baseline_ai.ps1:41-51` aggregates every `test result:` summary produced by the run and asserts passed, failed, and ignored totals; 26 and 10 are hard-coded and only `ExpectedPassed` is a parameter. It then requires exact set equality against the committed 26-name baseline (`:52-62`) and requires a nonzero cargo exit (`:63`) before resetting `$LASTEXITCODE`, so a fully green suite cannot masquerade as a matched baseline. It also refuses to proceed when no failure block is parsed at all (`:30-36`).

Both workflows drive it clean first and patched second, in that order: `ci_ai.yml:85-88` clean at the 3557 default, `ci_ai.yml:89-95` applies the patch, `ci_ai.yml:102-106` patched at 3570. The release path is the same shape at `release_ai.yml:133`. The 3,557 to 3,570 delta is stated in `VERIFICATION_ai.md:23` as 13 added passing tests with no new failure and no new ignored test, which is exactly what the two gate invocations enforce. The claim in the review request matches the committed gate.

## Item 2: post-turn mouse-capture handoff

This is the newest behavior and the reason for a fresh verdict, so I traced it end to end in the patch rather than through the evidence documents.

The decision function is pure and single-sourced:

```
should_release_mouse_capture_after_turn(task_was_running, task_is_running, composer_is_empty)
    = task_was_running && !task_is_running && composer_is_empty
```

(patch lines 2022-2028). Its only caller samples `task_was_running` before the `select!` in the main run loop and re-reads both live predicates after the event is dispatched (patch lines 51, 59-65, landing at `codex-rs/tui/src/app/startup.rs:707-718`). `composer_is_empty` resolves to `chatwidget.rs:1691-1693`, which is `bottom_pane.composer_is_empty() && !bottom_pane.is_in_paste_burst()`, so a draft arriving as a mid-flight paste burst is treated as non-empty. That is the conservative direction.

Both halves of the required contract hold:

- **Completed transcript is selectable with an empty composer.** On the running-to-idle transition with an empty composer, `release_mouse_capture_for_scrollback` clears `ENABLE_MOUSE_INPUT` (patch 2055-2062, `windows_console::set_mouse_capture`, patch 2214-2228), so VS Code owns the pointer immediately without needing a prior wheel event. That is precisely the regression being fixed.
- **An existing draft retains composer click and drag editing.** The `composer_is_empty` conjunct blocks the release, and the truth table `completed_turn_releases_capture_only_for_an_empty_composer` (patch 1920-1932) pins all four combinations, including the two that must not release.

Restoration is intact: `prepare_mouse_capture` restores capture on any `Key` or `Paste` event (patch 2101-2105), `recover_after_caught_panic` and the external-program path resynchronize the flag (patch context around 2130-2145), and `restore_common` unconditionally emits `DisableMouseCapture` on teardown.

Surrounding routing was checked rather than assumed. `Event::Mouse` is now translated in `event_stream.rs` and the catch-all `_ => None` arm is gone (patch 2095-2100), so a future crossterm variant fails at compile time instead of being dropped. Every one of the ten upstream files that owns an event loop appears in the patch and calls `prepare_mouse_capture_for_non_composer_event`: `cwd_prompt.rs`, `external_agent_config_migration/mod.rs`, `external_agent_config_migration/source.rs`, `model_migration.rs`, `onboarding/onboarding_screen.rs`, `resume_picker.rs`, `startup_draft.rs`, `startup_hooks_review.rs`, `update_prompt.rs`, plus `app/startup.rs`. `screen_size.rs` classifies `Mouse` with `Key` and `Paste` so a pointer event does not force a deferred draw size. When an overlay is open, `app.rs` routes the event to `handle_backtrack_overlay_event` before the composer match arm, so mouse input cannot leak into a hidden composer while a pager is displayed.

I also re-derived the byte-boundary safety of the selection code, because the patch was regenerated since it was last examined. `byte_pos_at_cell` computes `line_end = line.end.saturating_sub(1).min(self.text.len())` and slices `self.text[line.start..line_end]`. The wrapping module emits every range as `line_start..line_end + 1` or `line_end..line_end + 1` (`bottom_pane/textarea/wrapping.rs:136,138`), so `end >= start + 1` always holds and the subtraction cannot invert the slice. Every returned offset additionally passes through `clamp_pos_to_nearest_boundary`. No panic path found.

## Item 1: Rust safety and correctness

Nothing in `src/` blocks release. The paths I exercised in detail:

- **Windows process forwarding** (`src/process.rs`). `CreateProcessW` is called with an explicit application path and a separately built command line, never through a shell, so the `.cmd` re-parsing class of injection cannot occur. `quote_windows_argument` implements the `CommandLineToArgvW` backslash and quote rules, quotes the empty string, rejects embedded NUL, and deliberately does not quote `& | < > ^ %`, which the committed test asserts (`src/process.rs:294`). The child is created suspended, assigned to a job with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, and only then resumed, with `SuspendedChildGuard` terminating it if any setup step between creation and resume fails. The console control handler returns TRUE for Ctrl+C and Ctrl+Break so the child owns interruption, and for close, logoff, and shutdown it waits on the child handle for a bounded 3,000 ms. The handler-count protocol is correct in both directions: the handler increments `ACTIVE_CLOSE_HANDLERS` before it loads `ACTIVE_CHILD`, and `ConsoleHandler::drop` nulls `ACTIVE_CHILD`, unregisters, and then spins until the counter reaches zero, while declaration order guarantees `ConsoleHandler` drops before the process handle it protects. A handler that races in after unregistration reads a null pointer and skips the wait.
- **Discovery** (`src/discovery.rs`). Candidate names are ordered `.exe`, `.bat`, `.cmd`, `.ps1`, matching Windows resolution precedence. A `.bat` or `.ps1` that is not an official npm shim returns `UnsafeTarget` rather than being skipped, and `resolve_candidate` returns `Ok(None)` only for an extension-less file, which no Windows shell resolves from a bare command name. Errors propagate with `?`, so discovery fails closed on the first unsafe match instead of adopting a later PATH entry. The current directory and the owned shim directory are excluded before iteration. Official npm layouts are verified through `package.json` name plus an exact relative path (`is_expected_npm_codex_path`, `is_expected_npm_claude_path`), so an arbitrary executable cannot masquerade as a package member.
- **Native self-adoption** (`src/dispatcher.rs:52-110`). Only `TargetChanged` reaches the adoption path; a missing file becomes `NativeTargetMissing` and a version probe timeout falls back to the identity-validated native target with a visible warning, and only when a fallback is permitted (`native_timeout_can_fallback`). A lost adoption race surfaces as `StateChangedDuringOperation`, after which state is reloaded once and the winner revalidated rather than retried in a loop.
- **Selection and manifest checks** (`src/dispatcher.rs:130-315`). `cached_manifest_at` rejects a cache sequence above `highest_manifest_sequence` and rejects a verified manifest whose sequence disagrees with the cache, which is what makes rollback protection meaningful. Explicit enhanced Codex is fail-closed on every manifest error and on expiry; the default route degrades to verified native Codex with a warning; managed Claude warns and forwards unless the separate strict opt-in is enabled, and returns `Ok(false)` in every case, so no Claude route can ever select a non-native binary. Host drift is a warning only, correctly, because the VS Code version does not change the pinned Codex binary.
- **Enhanced artifact validation** (`src/dispatcher.rs:313-345`). The size and mtime fast path is gated on a nonzero recorded size and falls through to a full SHA-256 comparison, so a legacy record cannot skip verification. A failed integrity check on the non-explicit route degrades to the native target and revalidates it before launch.
- **Recursion** (`src/dispatcher.rs:113-129`). The target is canonicalized and rejected if it is inside the owned shim directory or is the running image itself. Nesting of the same kind is otherwise allowed, which is the correct narrowing.
- **State transactions** (`src/state.rs:258-352`). Writes hold an exclusive advisory lock on `state.lock` in the state root, taken with a bounded timeout and a reparse-point check on the root. `save_locked` writes a uniquely named temp file, `sync_all`s it, copies the previous state to `state.backup.json`, and then replaces atomically, so a lock-free reader on the launch path observes either the old or the new file and never a partial one.
- **Install, update, rollback, PATH, uninstall** (`src/installer.rs`). Install is idempotent and now distinguishes a newer verified bundle, printing the exact `cli-editor update --bundle` command (`:203-210`) instead of a generic already-installed line. PATH is restored only when installation recorded that it added the owned entry, and a pre-existing entry is preserved. Self-uninstall cannot be aborted by a shim removal failure: `remove_owned_shims_with` (`:1131-1144`) records the failure and continues the loop, `remove_or_defer` attempts POSIX-semantics deletion first and falls back to renaming the locked running image out of command resolution, and `cleanup_owned_root` performs one final enumeration so `report_owned_residue` (`:1223-1243`) names only what actually survived.

## Items 4 and 5: release automation, licensing, repository shape

The round-eleven build-flag defect is genuinely fixed, and fixed in the strong form. `scripts/build_release_ai.ps1:52-57` asserts that the pinned upstream `codex-rs/.cargo/config.toml` still contains the exact MSVC `rustflags` line before the patch is applied, so a future upstream bump that adds a flag fails the build rather than silently dropping it. `CARGO_ENCODED_RUSTFLAGS` is then set per build: the dispatcher build gets `+crt-static` plus the deterministic remap and `/Brepro` arguments (`:78-82`), and the upstream build additionally gets `link-arg=/STACK:8388608` (`:84-90`). Two post-link assertions make the outcome checkable rather than assumed: every shipped executable is rejected if it imports `VCRUNTIME*` or `MSVCP*` (`:151-159`), and both upstream executables are rejected unless the PE header reports `SizeOfStackReserve: 8388608` (`:160-166`). That closes the tested-versus-shipped divergence that was the blocking issue.

Signing handling is sound. The workflow default is `contents: read` (`release_ai.yml:21-22`); only `prepare` holds `contents: write` for draft enumeration (`:26-27`), and only `publish-draft` holds `id-token` and `attestations` (`:400-403`). The `sign-release` job declares no permissions block, so it inherits read, and it receives the seed only through the job `env` (`:378`), which the finalizer reads as `$env:` rather than through any interpolated script body. Signing runs after independent rebuild parity, so no upstream compilation ever coexists with the secret. `compatibility/public-key.hex` still validates its production fixture in `src/compatibility.rs`, and `scripts/finalize_release_ai.ps1` fails closed on any other key.

The publication gate is honest and now complete on the credential axis: `scripts/check_publication_candidate_ai.ps1:29` covers PEM private-key headers, classic `gh[pousr]_` tokens, fine-grained `github_pat_` tokens, `sk-` keys, and `AKIA` identifiers, alongside absolute local path, email address, binary content, reparse point, oversized file, oversized candidate, and duplicate current review checks. The measured candidate has 105,569 bytes of headroom under the deliberate 640 KiB budget, so publishing this report cannot cross it. `CASE_STUDY_ai.md:32` now cites that budget rather than an incidental measurement.

Licensing and presentation are in order: `LICENSE`, `NOTICE`, and `THIRD_PARTY_NOTICES_ai.md` are copied into the bundle, both frozen `cargo-about` reports are required to exist inside the ZIP, and `README.md` carries an explicit non-affiliation boundary and an honest statement that v0.1 is not Authenticode-signed.

## Findings

No blocker, high, or medium finding. Four low items follow; none of them changes routing, signature, update, rollback, uninstall, or release-safety behavior, and none needs to precede the first draft release.

### Low

**L-1. The turn-completion capture release is the only mouse-capture path with no recorded alternate-screen policy.**
`patches/codex/rust-v0.148.0/0001-desktop-composer.patch` lines 712-718 as applied to `codex-rs/tui/src/app/startup.rs`.

The other two entry points state a policy explicitly. `prepare_mouse_capture_for_event` refuses to release while the alternate screen is active (patch 2091-2096), and `prepare_mouse_capture_for_non_composer_event` deliberately releases even then, pinned by `non_composer_wheel_releases_capture_even_on_alt_screen` (patch 1963-1975). The turn-completion path calls `release_mouse_capture_for_scrollback` directly and therefore inherits neither. Failure scenario: a user presses Esc twice during a running turn to open the transcript overlay, which enters the alternate screen; the turn then finishes with an empty composer, and capture is dropped while a full-screen pager is displayed, where there is no host scrollback for the pointer to act on. It self-heals on the next keystroke and I could not turn it into data loss or a wrong launch, which is why it is low rather than medium. It matters mainly as a maintenance hazard: `PATCH_REVIEW_ai.md` requires mouse-capture disposition to stay single-sourced, and a third path with an unstated policy is what erodes that on the next upstream bump. Recommended correction: make the policy explicit at the call site, either by guarding with `!tui.is_alt_screen_active()` or by adding a comment stating that the release is intentional there, and extend the truth-table test with the chosen case.

**L-2. Uninstall residue guidance omits that the file is already queued for removal at the next restart.**
`src/installer.rs:1241` with `:1319-1322`.

When the running image is locked, `defer_delete` renames it and then calls `MoveFileExW(pending, NULL, MOVEFILE_DELAY_UNTIL_REBOOT)`, so in the common case Windows deletes it at the next boot without user action. `report_owned_residue` nonetheless prints only "delete final inert residue after this command exits", and it prints the same sentence whether that queueing succeeded or failed, because the result is discarded. A user is told to perform a manual step that is usually unnecessary, and a user whose queueing actually failed gets no stronger signal than one whose did not. Recommended correction: propagate whether the delayed-delete queue succeeded and say either "it is removed automatically at the next restart, or you can delete it now" or "queueing failed; delete it after this command exits".

**L-3. `README.md` recommends a key that has a side effect.**
`README.md:21`.

The line "Press `Esc` before clicking if you only want to restore capture without editing the prompt" is correct that any key restores capture, but with an idle turn and an empty composer, Esc is exactly the condition that primes Codex backtracking: `handle_backtrack_esc_key` returns early only when the composer is non-empty, and otherwise calls `prime_backtrack`, which shows the "Esc again to edit previous message" hint; a second Esc opens the backtrack preview overlay. So the documented gesture is the one gesture in the idle empty-composer state that changes visible application state. Recommended correction: name a genuinely inert key, or keep Esc and add that it also primes the backtrack hint.

**L-4. The review artifacts are the only publication-candidate files carrying em dashes and en dashes.**
`CLAUDE_IMPLEMENTATION_REVIEW_ai.md` and the three archived reports under `docs/reviews/`.

Every Codex-authored document in the candidate is clean; the four Claude reports are not, at 33, 34, 30, and 20 occurrences respectively before this replacement. If the workspace punctuation rule is meant to reach the published repository, the review artifacts are the standing exception to it. This report is written without them. Recommended correction: a one-line decision, either accept that archived independent reports are preserved verbatim as evidence, which is a defensible reason not to edit them, or normalize them and record that they were normalized. Do not silently rewrite them, since their value is that they are the reviewer's own words.

## Residual validation gaps (release gates, not defects)

1. **Live retest of the transcript-selection handoff.** `.handoff.md` still lists this as outstanding. The truth table and the wheel regression prove the decision function and the flag transitions, but `windows_console::set_mouse_capture` is `#[cfg(all(windows, not(test)))]`, so no automated test touches the real console mode. Capture is armed through crossterm's DECSET path in `set_modes` and released through the console-input `ENABLE_MOUSE_INPUT` bit, which are not the same mechanism under ConPTY. Confirm on a rebuilt CLI in both VS Code's terminal and Windows Terminal: select completed transcript text with an empty composer, then confirm a non-empty draft still accepts click placement and drag selection.
2. **Signing seed provisioning or key rotation.** No release can be produced until this exists. Correctly escalated; no ignored secret was read during this review.
3. **First GitHub Actions execution and draft release.** Every workflow gate here is structural. Bit-for-bit rebuild parity in particular has never run, and its inputs changed with the build-flag fix.
4. **Isolated clean-machine install acceptance.** The hosted lifecycle job covers the sequence, but on a runner that already has the VC++ redistributable; the new static-CRT assertions are what make that gap small rather than the acceptance run itself.
5. **The repository still has no commits.** `git archive HEAD` in `scripts/build_release_ai.ps1` and `git ls-files --cached` in the publication gate are currently exercised only against the 55-file untracked candidate. Re-run both after the initial commit.
6. **Hosted runner budget.** Each patch and preflight job compiles and runs the full `codex-tui` suite twice within 360 minutes, and each build job rebuilds at `codegen-units=1`. Plausible, unmeasured.

## Escalations for the user

- **Signing key provisioning or rotation**, and **creating the public repository**, are external authorization gates.
- **Platform-recognized `SECURITY.md` and `CONTRIBUTING.md` filenames.** GitHub links a security policy and a contributing guide only from those exact names in the root, `docs/`, or `.github/`. The workspace naming rule requires the `_ai` suffix, and Codex correctly declined to create conflicting aliases without authorization. This needs one decision: grant a named exception for these two reserved filenames, as already applies in practice to `README.md`, `LICENSE`, and `NOTICE`, or accept that the Security tab reads "no security policy" at publication. Enabling private vulnerability reporting is a separate setting either way.

## Assessment

The tree is release-ready as an implementation. The launcher's security-relevant boundaries all hold under direct reading: no shell is ever interposed between the dispatcher and a native CLI, argument quoting follows the documented parsing contract, discovery fails closed on the first unsafe match, manifest verification binds signature, sequence, and expiry together, Claude is never routed to anything but a verified native executable, and uninstall cannot strand a user with a mutated PATH and a live shim.

The change that prompted this round is correct and well bounded. The decision to release capture is a three-input pure function with an exhaustive test, its only caller samples the transition honestly, both the empty-composer and the draft cases behave as the contract requires, and the surrounding routing changes preserve the exhaustiveness guard that makes the next crossterm bump a compile error instead of a silent drop. `DESKTOP_COMPOSER_BEHAVIOR_ai.md` and `README.md` describe the new behavior accurately, including the tradeoff it imposes, rather than presenting it as free.

The round-eleven blocker is closed in the strong form, with a preflight assertion on the pinned upstream configuration and two post-link assertions on the produced binaries, so the tested and shipped configurations no longer diverge and a regression would fail the build rather than reach a user.

What remains is execution evidence, not implementation work: the live retest, the signing seed, and the first hosted run. Those are properly release gates. The four low items are independent polish and can land alongside or after the first draft.

VERDICT: APPROVED
