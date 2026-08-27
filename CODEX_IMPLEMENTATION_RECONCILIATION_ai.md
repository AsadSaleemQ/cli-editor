# Codex implementation reconciliation

Date: 2026-08-25

This document consolidates every finding against the live tree. To keep the repository lean, the public immutable review window contains the three prior completed rounds; earlier findings and retired exact reports remain fully dispositioned here. `CLAUDE_IMPLEMENTATION_REVIEW_ai.md` is always the fresh-verification destination, and approval is never inferred after subsequent edits.

## Blocker and high findings

| Finding | Resolution | Evidence |
|---|---|---|
| B1 invalid audit artifact | OAuth was restored, the complete `CHANGES_REQUIRED` review was captured, and its actionable findings are consolidated here and independently rechecked in later rounds. | This finding-by-finding reconciliation; independent review rounds eight through ten. |
| H1 nested same-kind commands blocked | Removed the inherited same-kind depth rejection. Dispatch now rejects only a target that resolves to the current dispatcher or its owned shim directory. | `src/dispatcher.rs`; `recursion_guard_rejects_only_owned_shim_targets`. |
| H2 exact VS Code host gate | Exact Codex compatibility remains fail-closed. An unlisted VS Code host now emits a warning and continues because it does not change the pinned Codex binary. | `src/dispatcher.rs`; explicit-host-drift regression test; current docs. |

## Medium findings

| Finding | Resolution | Evidence |
|---|---|---|
| M1 enhanced binary rehashed every launch | `ReleaseRecord` stores size and modification time. Dispatcher and doctor use that metadata fast path and hash only after metadata changes or for a legacy record. | `src/state.rs`, `src/dispatcher.rs`, `src/doctor.rs`. |
| M2 `claude cli-editor` had no effect | Explicit Claude invocation enters signed managed validation, but validation drift/unavailability now warns and forwards to unchanged native Claude. Fail-closed behavior requires the separate strict-default opt-in. `claude -- cli-editor` remains the literal escape. | `src/dispatcher.rs`; explicit unavailable/unlisted-Claude, strict-default, and argument-fidelity tests; READMEs. |
| M3 scroll capture leaked across non-composer screens | Added non-composer mouse handoff and invoked it from onboarding, resume, cwd, migration, update, startup-hook, external-agent-migration, and startup-draft loops. Wheel input releases capture and is consumed; keyboard/paste restores it. | Pinned Codex patch; `non_composer_wheel_releases_capture_even_on_alt_screen`. |
| M4 baseline gate asserted names only | The gate aggregates every Cargo test summary and asserts pass, fail, and ignored totals in addition to the exact failure-name set. It resets the native command exit status after the expected failing baseline is verified. | `scripts/check_codex_tui_baseline_ai.ps1`; clean expected 3,557 and patched expected 3,570. |
| M5 dependency license texts absent | The release builder runs pinned `cargo-about` for the dispatcher and upstream workspaces and requires both generated HTML license reports inside the ZIP. | `about_ai.toml`, `about_ai.hbs`, `scripts/build_release_ai.ps1`, release workflow. |
| M6 dispatch matrix untested | Added direct tests for shim-token stripping and delimiter escape, explicit Claude, strict Claude, exact Codex behavior, host drift, default fallback, and manifest grace. | `src/dispatcher.rs`; launcher suite has 66 passing tests. |
| M7 sequential release builds likely timed out | Primary and reproducibility builds are independent 360-minute jobs with Cargo/tool caching. A parity job compares their entire artifact inventories, and a separate isolated Windows job exercises the downloaded bundle lifecycle and exact PATH restoration before draft publication. | `.github/workflows/release_ai.yml`. |
| M8 code-mode host unproven | Both CI and release preflight now run an MSVC `cargo check --locked -p codex-code-mode-host` before signing builds. The release builder also builds and smoke-probes the executable. | CI and release workflows; local gnullvm check reached the Rusty V8 download boundary, so hosted MSVC remains authoritative. |

## Low findings

| Finding | Resolution |
|---|---|
| L1 unused missing-artifact error | Bundle verification now constructs `MissingReleaseArtifact` before reading a declared file. |
| L2 inaccurate reparse cleanup claim | Documentation now states cleanup does not traverse reparse directories and may remove or defer the reparse entry itself. |
| L3 pre-existing PATH entry removed | Uninstall restores/removes PATH only when installation recorded that it added the owned entry. |
| L4 doctor false-failed from extracted bundle | Doctor omits current-directory precedence for the `cli-editor` self-command while retaining precedence checks for managed native commands. |
| L5 active rollback comparison mismatch | Active and candidate release directories are canonicalized before comparison. |
| L6 rollback claimed shim restoration | Documentation now states rollback deliberately retains the newest dispatcher shims and switches the verified payload/cache/state. |
| L7 literal `cli-editor` escape undocumented | Both READMEs document `codex -- cli-editor` and `claude -- cli-editor`. |
| L8 build flags overwrote upstream flags | The release builder preserves the pinned 8 MiB Codex stack reservation and static CRT while adding deterministic remap and `/Brepro` arguments. It asserts the pinned upstream config before patching, then inspects every shipped PE for dynamic MSVC-runtime imports and both upstream executables for the 8 MiB stack header. |
| L9 release not gated and detached clone unsafe | Release preflight runs dispatcher fmt/clippy/tests, applies the pinned upstream patch, refreshes only the known lock anomaly, and checks code-mode host before build jobs. Release source is checked out directly rather than cloned from a detached local worktree. |

## Round-three findings

| Finding | Resolution | Evidence |
|---|---|---|
| H-1 explicit Claude always strict | Removed explicit invocation from the strictness decision. `claude cli-editor` and `cli-editor run claude` now validate, warn, and forward on an unlisted/missing/corrupt/expired manifest unless the user separately enabled a strict managed default. | `src/dispatcher.rs`; unavailable and unlisted explicit-Claude regressions; strict-default regression. |
| M-1 public review placeholder | Replaced the quota-error placeholder with Claude's complete round-three report; its actionable findings are consolidated here and rechecked by later independent review rounds. The publication gate now requires a substantive current report containing an exact verdict and rejects known authentication/quota prefixes. | This reconciliation; later independent reviews; `CLAUDE_IMPLEMENTATION_REVIEW_ai.md`; `scripts/check_publication_candidate_ai.ps1`. |
| M-2 unsupported npm-shaped launcher aborts all installation | Missing/malformed official package metadata now produces `UnsupportedLauncher` with an actionable message. Any per-CLI discovery failure leaves that CLI unmanaged with a warning while installation proceeds for another verified CLI; discovery never skips to a later PATH match. | `src/error.rs`, `src/discovery.rs`, `src/installer.rs`; missing-package regression. |

Round-three low findings were also dispositioned:

- L-1 is the documented control grammar: the optional `--` immediately after `cli-editor` is consumed as a separator. A literal native leading delimiter is forwarded as `cli-editor -- -- ...`; no ambiguous silent rewrite remains undocumented.
- L-2 is fixed by canonicalizing each automatic rollback candidate before comparing it with the canonical active directory.
- L-3 is intentionally unchanged: when installation did not add the PATH entry, uninstall must preserve that exact pre-existing user setting. Removing it would violate rollback fidelity; owned files are still removed and any pre-existing entry retains its pre-install value.
- L-4 now searches PATH only for the documented PowerShell precedence model and no longer fabricates current-directory precedence for Codex or Claude.
- L-5 now prints the machine-scope precedence risk at install success and requires verification in a new PowerShell terminal; hosted acceptance remains the release gate.
- L-6 now derives the builder's `CodexVersion` and the published tag suffix from the same validated prepare output.
- L-7 now aggregates every Cargo failure-name block while continuing to aggregate all test summaries.
- L-8 release preflight now executes the exact clean and patched full TUI count/name gates itself before either build can reach signing.
- L-9 wraps the private seed string and decoded bytes in zeroizing buffers, clears the stack seed immediately after key construction, and enables `ed25519-dalek` key zeroization.

## Additional pre-publication hardening

- Official `@anthropic-ai/claude-code` npm `.cmd`/`.ps1` shims now resolve through exact package metadata to `bin\claude.exe`; arbitrary script launchers remain rejected. The approved identity family can self-adopt only in place.
- Discovery now fails closed on the first unsafe PATH match instead of silently adopting a later command, preserving the native command-resolution boundary; a regression test covers this case.
- `status` now surfaces the latest adoption for each CLI, and `doctor --json` emits the bounded durable adoption history; serialization coverage prevents the audit trail from becoming invisible.
- Hosted lifecycle acceptance now measures 30 native and wrapped launches for both CLIs and blocks the draft release if p95 dispatcher overhead exceeds 50 ms.
- Default selection now rejects a requested CLI that was not installed, and `default all` applies atomically only when both native routes exist.
- Native-target identity failures now include the exact explicit repair command instead of leaving recovery implicit.
- Windows child launch now arms a pre-resume termination guard, preventing an orphaned suspended native CLI if Job Object, console-handler, or resume setup fails.
- Release-dispatch inputs are passed only through environment values, validated before secret-bearing jobs, checked against `Cargo.toml`, and propagated as sanitized prepare outputs, closing PowerShell/GitHub-token injection paths.
- Both rebuilds consume the same prepared `SOURCE_DATE_EPOCH`, and `cli-editor --version` now carries a regression-tested unofficial/non-affiliation marker.
- The release builder now serializes `compatibility-manifest.json` before deriving its SBOM digest; the signer revalidates the prepared version, sequence, issue time, and expiry, and the finalizer fixture enforces that build-to-sign handoff before exercising negative inventory checks, signing, and deterministic packaging.
- Independent compilation and complete unsigned inventory parity now finish before a dedicated protected `sign-release` job receives the Ed25519 seed. Upstream builds have read-only tokens and never coexist with the signing secret; only the draft publisher receives write/id-token/attestation permissions. All three uploads explicitly include the controlled hidden `.artifacts` root, matching the pinned upload action's default-exclusion behavior; the signed upload excludes internal release tools. A checked-in development-key fixture now exercises the isolated finalizer end to end in CI without accessing the production seed. Unsigned and signed bundle/top-level inventories are exact allowlists, and the finalizer rejects duplicate or missing manifest artifact names, so an unexpected deterministic file or ambiguous manifest fails before signing or upload.
- The release workflow now downloads the primary ZIP on a separate Windows runner, installs pinned official Codex and Claude packages, exercises dry-run/install/status/doctor/both bare default routes/both explicit routes/restore/both restored native routes/uninstall, and requires exact raw user-PATH equality plus no owned residue before draft publication.
## Round-four approval and low cleanup

Claude's round-four report ended `VERDICT: APPROVED` and explicitly classified all remaining items as low. Before publication, Codex also cleared those items:

- L1: the current review was replaced with the complete approval report, and the public gate now rejects a current review byte-identical to any numbered historical review. The older exact report was later retired from the public rolling window after its findings were consolidated here.
- L2: uninstall prints when it preserves a pre-existing, unowned shim PATH setting; both lifecycle documents qualify exact restoration accordingly.
- L3: `default all` now preserves Claude strictness unless a strict flag is explicit, while strict flags on Codex-only defaults fail visibly; regression coverage proves both behaviors.
- L4: the CI patch and release preflight jobs have explicit 360-minute limits and pinned Cargo registry/git caches keyed to the upstream commit.
- L5: the validated GitHub repository slug is a prepare output consumed by both release builders and draft publication; signed manifest URLs and the publish target now share one source.
- L6: the fixture no longer reads stale `$LASTEXITCODE` after PowerShell-script invocations; terminating errors remain governed by `$ErrorActionPreference = 'Stop'`.

## Round-five findings

Claude's round-five verification identified one medium cold-start risk and six lows. All are resolved in the current tree:

- M-1: native CLI identity probes and freshly copied enhanced release artifacts use the bounded 60-second cold-artifact budget for install, update, and every rollback candidate. Timeout errors include the elapsed budget. Probe output uses a capped temporary file rather than reader threads, so a descendant inheriting stdout cannot extend either timeout by keeping a pipe open. Regressions lock the release/native budget relationship and the 64 KiB capture/cleanup bound.
- L-1: shim mode derives from `current_exe()` first and uses `argv[0]` only when the OS path cannot be obtained.
- L-2: repeat install now reports that existing state was revalidated and republished instead of claiming zero owned-file changes.
- L-3: the front-page README now carries the same qualified PATH restoration contract as the technical lifecycle docs.
- L-4: console teardown nulls the active handle, unregisters the handler, and waits for every handler that observed the handle before the owning handle can close.
- L-5: doctor names the raw-registry check `user PATH entry present`; separate per-command checks retain the actual precedence assertion.
- L-6: heavy release preflight now depends on fast validated `prepare`, so invalid dispatch inputs cannot burn a 360-minute runner.

## Additional pre-final-review hardening

Codex's final safety audit also removed inherited-pipe joins from version probes and bound state-owned paths to the installation root. Probe output now uses a capped temporary file, so descendant handle inheritance cannot extend the 60-second budgets. State load/save rejects redirected shim, release, and manifest paths plus reparse-point owned directories before any command can mutate PATH or remove files; the common-path check stays lexical and canonicalizes only the Windows short/verbatim fallback, avoiding unnecessary launch-time filesystem work. Cleanup independently rejects a reparse-point root and recursive directory entries. Missing-CLI repair also removes its newly copied shim if state publication fails, preserving the same compensating-rollback boundary as installation and update.
## Current validation

- Dispatcher formatting and warnings-denied clippy passed; all 66 tests passed in both debug and optimized release-mode runs.
- The full 66-test debug suite also passed with test-harness thread counts 1, 2, and 8, confirming serial and concurrent scheduling.
- Release dispatches now share one non-cancelling concurrency group; after the prior run finishes, preparation enumerates all existing GitHub releases and drafts and requires the requested signed-manifest sequence to exceed their maximum. Its exact PowerShell block passed parsing plus empty, greater-than, and equal-sequence mock cases, and the modified workflows pass Actionlint 1.7.12. CI uses a separate ref/PR-scoped cancelling group so superseded six-hour patch validations do not waste hosted capacity.
- `Cargo.toml` declares `cli-editor` as the default binary, so the public contributor path `cargo run -- --version` is unambiguous despite the separate isolated signer binary.
- Patch is 35 files and 94,244 bytes, SHA-256 `6a87f803bd7f6175c47419ecb0f254c0f62063239ce6007a01c670b259cff08c`.
- Forward apply, reverse apply, and whitespace checks passed after patch regeneration.
- Patched Codex TUI aggregate: 3,570 passed, 26 known baseline failures, 10 ignored.
- Clean Codex TUI aggregate: 3,557 passed, the same 26 known baseline failures, 10 ignored.
- All six pinned GitHub Action SHAs were verified live against their documented version tags.
- Pinned Actionlint 1.7.12 passed both workflows locally, and CI repeats it after verifying the tool archive against a hard-coded SHA-256.
- The 57-file candidate passes the size/privacy/supply-chain checks; the final nonduplicate-current-review assertion awaits the complete fresh exact-tree report. The repeatable local and CI gate is in `scripts/check_publication_candidate_ai.ps1` for credential patterns, private-key headers, email addresses, absolute local paths, binaries, reparse points, files over 1 MiB, and a total candidate over the deliberate 640 KiB lean-source budget.
- Local pinned cargo-about renders for both workspaces passed. Public GitHub CI is now green on commit `574a0d0`; the fresh exact-tree Claude verdict, signed draft release, and isolated downloaded-artifact lifecycle acceptance remain explicit gates.
- The committed verification public key has a valid production-path fixture, but the corresponding private seed is unavailable. Rotation and GitHub secret provisioning require explicit user authorization.
## Round-six findings

Claude's fresh round-six review identified two medium and four low findings. The current tree resolves the shipped-code findings as follows:

- M-1: first discovery and in-place native adoption now use the same bounded 60-second cold-artifact budget as enhanced release probes. If a changed native target times out after its canonical path, package root, expected package family, and executable shape revalidate, default Codex and all Claude routes warn and force the native target for that launch; explicit enhanced Codex still fails visibly. The validated-native result is reused, removing the duplicate canonicalize/metadata pass noted in L-2.
- M-2: update compatibility checks are conditional on Codex being managed. Claude-only installations can receive dispatcher, Claude-route, manifest, and payload updates; a regression proves no-Codex state succeeds while an incompatible managed Codex still fails closed.
- L-1: install now re-verifies all three copied executables against the signed manifest before probing or publishing state, then records the manifest-declared enhanced digest and size instead of re-hashing a potentially corrupted copy as its own baseline.
- L-3: the rolling immutable review window lives under `docs/reviews/`; the duplicate-review publication assertion follows that path, the technical guide links the review history, and the stale dispatcher count is corrected.
- L-4: the existing signed state-level regression was renamed to state its full scope: update stages and activates a signed sequence, preserves the prior release, then rollback re-verifies and activates the prior signed sequence while retaining the monotonic high-water mark. It runs in hosted CI alongside the separate production-signed install/default/restore/uninstall lifecycle job. A production-signed update/rollback on a downloaded draft remains an explicit release gate because synthesizing another production sequence inside CI would either expose the seed to an upstream-building job or publish test-only production manifests.

## Round-seven findings

Claude's fresh round-seven review found one medium release-control defect and three low hardening items. The current tree resolves each one:

- M-1: the `prepare` job alone now receives `contents: write`, allowing its release enumeration and manifest-asset reads to include drafts while build and preflight jobs retain read-only contents access. The first hosted release gate still requires an empirical repeated-sequence dispatch to fail during preparation.
- L-1: the publication verdict accepts LF or CRLF, and duplicate-current-review comparison normalizes CRLF to LF before equality so a Windows re-save cannot bypass or falsely fail either assertion.
- L-2: update now mirrors install by recording the signed manifest's declared enhanced-artifact digest and size after staged verification; the signed update regression asserts both fields against that manifest declaration.
- L-3: composer and non-composer mouse-capture entry points now delegate to one event-disposition implementation with an explicit alt-screen policy. The upstream-bump checklist requires this event-kind handling to remain single-sourced.
## Round-eight findings

Claude's round-eight review ended `VERDICT: APPROVED` with three low hardening findings. Codex resolved all three before the final exact-tree review:

- L-1: `CASE_STUDY_ai.md` now reports the measured patch size used by every other provenance record; the latest regenerated patch is 94,244 bytes.
- L-2: rollback now records the verified retained manifest's declared enhanced-artifact digest and size, matching install and update; the signed update/rollback regression asserts those rollback fields.
- L-3: release preparation downloads each prior manifest by release tag with `gh release download` into a dedicated runner-temp file, keeps stderr separate from JSON, and fails closed with the asset identifier and CLI error. The repeated-sequence hosted gate remains required.
## Round-nine findings

Claude's round-nine review identified one high self-uninstall defect and three low hardening items. The current tree resolves all four:

- H-1: shim removal failures no longer abort state deletion after PATH mutation. Windows cleanup first attempts POSIX-style namespace deletion; for a locked running image it renames the executable to a unique `.pending-delete.<pid>.<nonce>.exe`, immediately removing the command interception, and best-effort queues the renamed file for restart deletion. Queue failure is nonfatal and reports the exact inert residue for removal after exit. Unit coverage proves a forced shim-removal error cannot abort the remaining shim loop, proves normal POSIX deletion, and runs a copied test executable while validating that the in-use fallback removes its original command path. Hosted lifecycle acceptance now invokes final uninstall through the installed shim, requires state and all three active shim names to be gone, permits at most one explicitly named running-image residue, removes it after process exit, and still requires exact PATH restoration and no owned root.
- L-1: when two first-launch adoptions race, the loser now reloads state once after `StateChangedDuringOperation`, validates the winning native metadata, and continues. A regression reproduces that race and proves the winning record is used.
- L-2: `repair --adopt-native` now copies the newest installed dispatcher from `state.shim_directory\cli-editor.exe`, never from a potentially rolled-back payload directory.
- L-3: the public immutable review window rolls forward to rounds seven through nine; older exact reports moved to ignored `.trash/` after their findings were consolidated here, preserving useful independent evidence without consuming the lean candidate budget.
## Round-ten findings

Claude's round-ten review ended `VERDICT: APPROVED` and classified seven remaining items as low. The current tree clears six and explicitly dispositions the one workspace-policy conflict:

- L-1: missing recorded native executables now return `NativeTargetMissing` in the launcher-error class with the missing path, reinstall/repair guidance, and uninstall escape. Dispatcher and recorded-identity regressions cover both validation paths, and doctor receives the same actionable detail.
- L-2: `install` detects invocation through the owned `cli-editor.exe` shim before bundle discovery and returns the specific already-installed/extracted-release guidance. A canonical-path regression distinguishes the installed shim from an external executable.
- L-3: both release license renders now pass `cargo about generate --frozen`, removing network and ClearlyDefined variability. Local frozen renders passed for the dispatcher (114,925 bytes) and pinned Codex workspace (1,138,566 bytes).
- L-4: the mandatory workspace naming rule requires AI-authored Markdown to retain `_ai`, so Codex did not silently create conflicting `.github/SECURITY.md` or `.github/CONTRIBUTING.md` aliases. `README_ai.md` directly links both public guides, and enabling GitHub private vulnerability reporting remains a publication gate. A platform-recognized filename exception requires explicit user authorization.
- L-5: release notes now qualify exact PATH restoration to an unchanged setting owned by CLI Editor and explicitly preserve later edits.
- L-6: the publication gate now has a deliberate 640 KiB lean-source budget rather than an accidental 512 KiB cliff, while the measured candidate remains well inside that budget.
- L-7: the publication text-hygiene record now consistently reports 55 candidate files.
## Round-eleven findings

Claude's round-eleven review found one high, one medium, and five low release-hardening items. The current tree resolves all seven without changing the product contract:

- H-1: release builds now preserve the pinned upstream 8 MiB stack reservation and static CRT instead of replacing those flags with deterministic flags. The builder asserts the exact pinned upstream MSVC configuration before applying the patch, and post-build PE inspection proves both upstream executables retain the 8 MiB reserve.
- M-1: the dispatcher release build explicitly enables static CRT too. Post-build import inspection rejects dynamic `VCRUNTIME` or `MSVCP` imports in every shipped executable.
- L-1: the portfolio size claim now references the deliberate 640 KiB source budget rather than a transient earlier measurement.
- L-2: self-uninstall suppresses stale intermediate delete names and performs one final owned-root enumeration, reporting only residue that actually remains after cleanup.
- L-3: release enumeration keeps `gh api` stderr in a dedicated runner-temp file, so diagnostics cannot corrupt successful JSON output.
- L-4: the publication credential scanner now detects fine-grained `github_pat_` tokens as well as classic GitHub tokens and the other existing credential forms.
- L-5: when `install` verifies a newer extracted bundle but finds an existing installation, it prints the exact `cli-editor update --bundle` command needed to activate that bundle; equal or older bundles retain the idempotent already-installed result.
## Round-twelve approval and priority-low cleanup

Claude's round-twelve report ended `VERDICT: APPROVED`, verified every round-eleven correction, and found six non-blocking low items. Before the final exact-tree review, Codex applied the two items Claude prioritized ahead of the first draft:

- L-2: the release builder now performs an explicit locked dispatcher dependency fetch before frozen license generation, ensuring a cold runner has dev-dependencies needed by `cargo metadata` instead of failing after both long builds.
- L-6: the reconciliation describes the rolling archive generically as the three most recent completed rounds, avoiding stale round-number text on future rotations.

Round-twelve L-1 (host-warning wording), L-3 (backup recovery), L-4 (future upstream rustflag-location hardening), and L-5 (verbatim-path presentation) remain explicitly approved post-draft hardening, not unreported defects. They do not alter current routing, signature, update, rollback, uninstall, or release-safety behavior.

## Post-round-twelve transcript-selection regression

The user found that assistant transcript text could not be selected immediately after a response. Typing or paste had restored terminal mouse capture, and only a later wheel event released it, so the earlier live selection check had accidentally depended on scrolling first.

The patched TUI now records whether a task was running before each event cycle. After the turn transitions from running to idle, it releases mouse capture only when the composer is empty. This gives completed transcript drag selection back to VS Code while preserving click placement and drag selection for an existing draft. Keyboard or paste still restores capture, and wheel handoff remains unchanged. The focused truth-table regression and the existing wheel regression passed; the complete patched aggregate passed at 3,570/26/10 against the unchanged clean 3,557/26/10 baseline. A fresh user live retest and exact-tree Claude verdict remain required.

## First hosted CI reconciliation

The public repository is live at `AsadSaleemQ/cli-editor`. Hosted execution found two portability gaps without exposing a product-runtime regression:

- Run `33045649893` showed that Windows checkout conversion changed byte-pinned patch/fixture inputs and that Linux PowerShell did not enumerate `.gitignore` through the original item lookup. `.gitattributes` now preserves LF for release-critical text, and the publication scanner resolves every Git-listed path from the repository root.
- Run `33045970130` proved the dispatcher and clean 3,557-test Codex baseline, then showed that Linux PowerShell requires `Get-Item -Force` for the dotfile and that the generic Rusty V8 MSVC URL returns 404 for Codex's pointer-compression/sandbox build.
- The fix pins both official OpenAI Codex Rusty V8 150.4.0 assets by name, byte length, and SHA-256; verifies the pinned upstream `Cargo.lock` version and `MODULE.bazel` URL/hash provenance; downloads atomically; and supplies both build-script environment paths in CI, release preflight, and reproducible release builds.
- Public run [`33047859125`](https://github.com/AsadSaleemQ/cli-editor/actions/runs/33047859125) on exact commit `574a0d0ea3ffc13fe395d0bc21e079183c4599b2` passed all three jobs. This includes Ubuntu publication/workflow lint, all 66 dispatcher tests and release/finalizer gates, clean 3,557/26/10 Codex baseline, official-V8 code-mode probe, and patched 3,570/26/10 aggregate.

These hosted fixes changed release scripts and workflows after Claude's prior approval. A new exact-tree Claude verdict is therefore still mandatory even though the changed public commit is fully green.
