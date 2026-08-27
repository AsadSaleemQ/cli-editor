# CLI Editor v0.1.0

Initial Windows x64 release of CLI Editor.

## Included

- Desktop-style Codex composer: touchpad/wheel scrolling, click placement, drag selection, Ctrl+A/C/X/V, Ctrl+Home/End prompt navigation, Shift selection variants, image paste, undo/redo, and a visible mouse cursor.
- Native Codex and Claude pass-through with direct `CreateProcessW` launch, console-control forwarding, Job Object cleanup, and exact exit-code propagation.
- Opt-in enhanced invocation plus persistent defaults, status/doctor, signed updates, compatibility fallback, explicit native adoption, and exact PATH rollback on uninstall when CLI Editor owned the unchanged setting, while preserving later edits.

## Compatibility

Validated baseline: Windows 11 x64, VS Code 1.134.0, Codex CLI 0.148.0, and native Claude Code 2.1.240. Claude Code is not modified or redistributed.

## Integrity

Verify the `.zip` against the adjacent `.sha256` file. The bundle also contains the exact Ed25519-signed compatibility manifest used by the dispatcher.

CLI Editor is an independent modified build and is not affiliated with or endorsed by OpenAI, Anthropic, or Microsoft.
