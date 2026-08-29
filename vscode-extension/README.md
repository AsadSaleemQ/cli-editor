# CLI Editor

Turn Codex CLI terminals into chat-style editors with smart clipboard paste and familiar shortcuts.

CLI Editor combines the VS Code extension in this package with an enhanced Codex CLI build from the [CLI Editor GitHub repository](https://github.com/AsadSaleemQ/cli-editor). The extension delivers terminal-level shortcuts; the enhanced Codex build adds desktop-style editing inside the prompt. Smart Paste also works in Claude Code terminals, but CLI Editor does not patch or redistribute Claude Code.

## Keyboard shortcuts

| Shortcut | Behavior | Scope |
|---|---|---|
| `Ctrl+A` | Select the complete editable prompt. | Enhanced Codex |
| `Home` / `End` | Move to the start / end of the current prompt line. | Enhanced Codex |
| `Ctrl+Home` / `Ctrl+End` | Move to the start / end of the complete prompt. | Enhanced Codex via this extension |
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

## Mouse and scrolling

| Input | Behavior |
|---|---|
| Click | Place the prompt cursor. |
| Click and drag | Select prompt text across lines. |
| Double-click / triple-click | Select a word / line. |
| Mouse wheel / touchpad | Hand scrolling back to VS Code terminal history; the next edit restores prompt interaction. |

## Install the complete product

Download the latest Windows release from [GitHub Releases](https://github.com/AsadSaleemQ/cli-editor/releases). The signed bundle installs the enhanced Codex build, launcher, and this extension as one reversible product.
