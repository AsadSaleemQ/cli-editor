# CLI Editor

**Turn Codex CLI terminals into chat-style editors with smart clipboard paste and familiar shortcuts.**

Click to place the cursor, select with the mouse or keyboard, copy and cut, paste text or images, undo and redo, and navigate long drafts like a desktop chat composer.

One repository ships the complete product: an enhanced Codex build, the `cli-editor` launcher, and the `asadsaleemq.cli-editor` VS Code extension. Claude Code is supported as a validated native pass-through and can use terminal-level Smart Paste, but CLI Editor never patches or redistributes Claude Code.

## Chat-style controls

### Keyboard shortcuts

| Shortcut | Behavior | Scope |
|---|---|---|
| `Ctrl+A` | Select the complete editable prompt. | Enhanced Codex |
| `Home` / `End` | Move to the start / end of the current prompt line. | Enhanced Codex |
| `Ctrl+Home` / `Ctrl+End` | Move to the start / end of the complete prompt. | Enhanced Codex via the extension |
| `Shift+Left` / `Shift+Right` | Extend or shrink the selection by one character. | Enhanced Codex |
| `Shift+Up` / `Shift+Down` | Extend the selection across lines. | Enhanced Codex |
| `Shift+Home` / `Shift+End` | Select to the start / end of the current line. | Enhanced Codex |
| `Ctrl+Shift+Left` / `Ctrl+Shift+Right` | Extend the selection by word. | Enhanced Codex |
| `Ctrl+Shift+Home` / `Ctrl+Shift+End` | Select to the start / end of the complete prompt. | Enhanced Codex |
| `Ctrl+C` | Copy selected prompt text; without a selection, preserve interrupt/cancel. | Enhanced Codex |
| `Ctrl+X` | Cut the selected prompt text. | Enhanced Codex |
| `Ctrl+V` | Paste clipboard text, or pass terminal-native image paste when no text is present. | VS Code terminal, including Codex and Claude Code |
| `Ctrl+Alt+V` | Force clipboard-image paste. | Enhanced Codex |
| `Ctrl+Z` | Undo the last prompt edit. | Enhanced Codex |
| `Ctrl+Shift+Z` | Redo the last undone prompt edit. | Enhanced Codex |
| `Backspace` / `Delete` | Delete the complete selected range. | Enhanced Codex |

### Mouse and scrolling

| Input | Behavior |
|---|---|
| Click | Place the prompt cursor. |
| Click and drag | Select prompt text across lines. |
| Double-click / triple-click | Select a word / line. |
| Mouse wheel / touchpad | Hand scrolling back to VS Code terminal history; the next edit restores prompt interaction. |
| Completed response selection | Select and copy terminal output normally when the prompt is empty. |

## Capabilities

### Launcher and CLI integration

- Run enhanced Codex explicitly with `codex cli-editor` or make it the default.
- Run Claude through the unchanged native executable with signed compatibility checks.
- Forward arguments, console-control events, exit codes, and process cleanup without shell-wrapper ambiguity.
- Inspect installation health with human-readable or JSON diagnostics.
- Adopt legitimate in-place native CLI updates while rejecting relocation or identity drift.

### Safe lifecycle management

- Verify enhanced artifacts and compatibility metadata with an embedded Ed25519 public key.
- Fail visibly for unsupported explicit enhanced requests and fall back to verified native Codex for default routes.
- Stage updates transactionally, retain signed prior releases, and support verified rollback.
- Restore native defaults at any time.
- Uninstall only owned files and PATH entries while preserving unrelated edits and pre-existing extensions.

See the [desktop composer guide](docs/DESKTOP_COMPOSER_BEHAVIOR.md) for the complete input contract.

## Install

Download the latest Windows x64 ZIP and adjacent `.sha256` file from [GitHub Releases](https://github.com/AsadSaleemQ/cli-editor/releases), verify the checksum, extract the archive, and run:

```powershell
.\cli-editor.exe install
```

CLI Editor installs its per-user launcher and, when VS Code is available, the bundled **CLI Editor** extension. Reload VS Code after installation. Named VS Code profiles must enable the extension in the profile used by the terminal.

Open a new terminal so it inherits the updated user PATH, then try:

```powershell
codex cli-editor
claude cli-editor
cli-editor status
cli-editor doctor
```

The source repository intentionally excludes compiled release executables. Cloning the repository is for development, not installation.

## Choose how each CLI launches

| Goal | Command |
|---|---|
| One enhanced Codex session | `codex cli-editor` |
| One validated native Claude session | `claude cli-editor` |
| Enhanced Codex by default | `cli-editor default codex` |
| Managed native Claude by default | `cli-editor default claude` |
| Configure both defaults | `cli-editor default all` |
| Restore native Codex and Claude | `cli-editor restore all` |
| Pass `cli-editor` literally to a native CLI | `codex -- cli-editor` or `claude -- cli-editor` |

Claude strictness controls whether an unlisted native Claude version is rejected or launched with a warning; it does not enable an enhanced Claude composer.

## Manage the installation

```text
cli-editor status
cli-editor doctor [--json]
cli-editor update --bundle DIRECTORY
cli-editor rollback [--release RELEASE]
cli-editor repair --adopt-native codex|claude
cli-editor uninstall
```

Updates are explicit and bundle-based; startup never blocks on a network download. See the [update and rollback contract](docs/UPDATE_AND_ROLLBACK.md) for recovery and ownership behavior.

## Compatibility and trust

The current validated baseline is Windows 11 x64, VS Code 1.134 and 1.135, Codex CLI 0.148.0, and native Claude Code 2.1.240 and 2.1.251. Enhanced Codex compatibility is exact and signed. Host-version drift may warn without changing the pinned enhanced binary; suspicious native-target or artifact changes fail closed.

Release assets are reproducibly built, hash-checked, provenance-attested, and finalized with a signing key that is never uploaded to GitHub. Windows may still show SmartScreen on an unsigned executable without established reputation.

## Documentation

- [Desktop composer behavior](docs/DESKTOP_COMPOSER_BEHAVIOR.md) — complete mouse, keyboard, clipboard, selection, and scrolling contract.
- [Technical guide](TECHNICAL_GUIDE.md) — architecture, process fidelity, compatibility, building, and release design.
- [Update and rollback](docs/UPDATE_AND_ROLLBACK.md) — update verification, retention, rollback, uninstall, and failure recovery.
- [Verification](docs/VERIFICATION.md) — supported baseline, automated gates, and acceptance checklist.
- [Security policy](SECURITY.md) — trust boundaries and vulnerability reporting.
- [Release notes](RELEASE_NOTES.md) — version-specific changes only.

## Project boundary

CLI Editor is an independent modified build and is not affiliated with or endorsed by OpenAI, Anthropic, or Microsoft. Codex source is used under Apache-2.0. Claude Code is neither modified nor redistributed.
