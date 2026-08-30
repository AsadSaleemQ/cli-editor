# Unreleased

## CLI Editor v0.1.3

- Removed non-Codex discovery, shimming, routing, defaults, diagnostics, and release compatibility.
- Upgrades retire legacy Claude routing state and its old shim while leaving the separately installed native CLI untouched.
- Restricted VS Code prompt navigation and Smart Paste to matching live enhanced-Codex sessions.
- Bumped the bundled CLI Editor extension to 0.3.2 with Codex-only commands, metadata, and documentation.

## CLI Editor extension

- Standardized the product, extension, VSIX, package, folder, and GitHub identity on **CLI Editor** (`asadsaleemq.cli-editor`, `cli-editor.vsix`, and `AsadSaleemQ/cli-editor`).
- Consolidated Terminal Smart Paste into CLI Editor so chat-style prompt editing, smart text/image paste, and familiar shortcuts ship from one extension and one repository.
- Preserved the former `terminalSmartPaste.paste` command as a compatibility alias while making `cliEditor.smartPaste` the canonical command.
- Added manifest/runtime contract coverage and explicit extension activation, background behavior, and compatibility documentation.

# CLI Editor v0.1.2

Windows x64 patch release of CLI Editor.

## Corrected

- Ctrl+Home and Ctrl+End now always reach the active terminal application instead of falling back to VS Code's scroll-to-top or scroll-to-bottom commands when process matching drifts.
- VS Code bridge tests cover both exact terminal sequences and the no-active-terminal case.

## Included

- Desktop-style Codex composer: touchpad/wheel scrolling, click placement, drag selection, Ctrl+A/C/X/V, Ctrl+Home/End prompt navigation, Shift selection variants, image paste, undo/redo, and a visible mouse cursor.
- Native Codex pass-through with direct `CreateProcessW` launch, console-control forwarding, Job Object cleanup, and exact exit-code propagation.
- Opt-in enhanced invocation plus persistent defaults, status/doctor, signed updates, compatibility fallback, explicit native adoption, and exact PATH rollback on uninstall when CLI Editor owned the unchanged setting, while preserving later edits.

## Compatibility

Validated baseline: Windows 11 x64, VS Code 1.134.0 and 1.135.0, and Codex CLI 0.148.0.

## Integrity

Verify the `.zip` against the adjacent `.sha256` file. The bundle also contains the exact Ed25519-signed compatibility manifest used by the dispatcher.

CLI Editor is an independent modified build and is not affiliated with or endorsed by OpenAI or Microsoft.
