# Verification

Date: 2026-08-29

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

CI run `33192184050` passed on commit `6df0b1d29303ce5eaf5132c4fbee9ce209b3bdff`. It includes the Windows regression that ensures VS Code's Node launcher receives a normal drive path rather than a verbatim `\\?\` path.

Release run `33194546859` passed from the same commit with manifest sequence 2. Its independent clean builds completed in 79 minutes 42 seconds and 65 minutes 26 seconds; the workflow then passed bit-for-bit artifact comparison and unsigned-build provenance attestation. The only annotation was GitHub's Node 20 action deprecation notice, not a product or build failure.

All six hosted subjects passed `gh attestation verify`: `cli-editor.exe`, `codex-enhanced.exe`, `codex-code-mode-host.exe`, `cli-editor-vscode.vsix`, the CycloneDX 1.5 SBOM, and the source archive. The SBOM contains 1,377 components. The two generated HTML license inventories are present at 114,924 and 1,138,565 bytes. The source archive contains exactly the same 57 files tracked at the release commit, with no build, cache, signing, review, or local-path entries. The SBOM and license reports contain no local path, username, email, or signing-secret references.

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

An earlier live VS Code terminal test confirmed touchpad/wheel scrolling, clipboard copy/paste, image paste, mouse cursor behavior, composer click/drag editing, and Ctrl+X/A/C/V. The later completed-turn capture release and conditional Ctrl+Home/End bridge still require a fresh live confirmation from the final candidate for empty-composer transcript selection, non-empty-draft editing, alternate-screen capture retention, and prompt-versus-terminal boundary routing.

## Signed-candidate lifecycle

The exact primary artifact from release run `33194546859` was downloaded and finalized locally without uploading the signing seed. The signed Windows ZIP is 110.46 MiB with SHA-256 `f72394dc740b234758b46bc9b56f739a19d895007140395f5e268a63c5032845`. The following checks passed from that ZIP:

- signature, manifest sequence 2, declared artifact inventory, sizes, and hashes;
- clean install and deterministic VS Code bridge installation as `asadsaleemq.cli-editor-vscode@0.1.0`;
- healthy `status` and `doctor --json` results for native Codex 0.148.0, native Claude 2.1.247, all three shims, command precedence, and the enhanced Codex artifact;
- explicit `codex cli-editor --version` and `claude cli-editor --version` routes;
- `default all --no-strict`, bare Codex and Claude routes, and `restore all` back to native defaults;
- fail-closed strict Claude behavior for unsupported 2.1.247, repeated-sequence rejection, and no-prior-release rollback rejection, each with exit code 125;
- byte-exact restoration of the original raw UTF-16 user PATH value and registry type after external uninstall;
- removal of owned `%LOCALAPPDATA%\CLIEditor` state and the owned VS Code extension, followed by native Codex and Claude command resolution;
- bounded five-sample p95 startup comparison: Codex shim was 17.8 ms below the native launcher measurement, while Claude shim added 12.2 ms. These small differences are within process-start measurement noise and show no material launcher penalty.

VS Code 1.135.0 was newer than the signed host validation set and produced the intended visible non-fatal warning before enhanced Codex continued. Claude 2.1.247 was newer than the signed Claude set and produced the intended warning before verified native pass-through; strict mode rejected it.

The first self-uninstall correctly removed command resolution and state but reported one locked inert shim queued for later deletion. A second lifecycle invoked uninstall from the external signed bundle, removed all owned state immediately, and proved byte-exact PATH restoration. This is functional but leaves the existing low-priority opportunity to make the queued-versus-manual residue message more specific.

Before a non-draft release:

1. complete the live mouse-capture and Ctrl+Home/End retest;
2. inspect the SBOM, unsigned-build provenance, and generated license reports for the final exact-HEAD candidate;
3. publish the corrected signed draft after resolving the older private draft/tag collision; and
4. repeat clean-machine acceptance if the final release commit changes executable inputs.

No corrected signed draft or live final-candidate UI acceptance is claimed until those gates execute. Reproducible-release parity and the local downloaded-artifact lifecycle above are complete for commit `6df0b1d29303ce5eaf5132c4fbee9ce209b3bdff`.
