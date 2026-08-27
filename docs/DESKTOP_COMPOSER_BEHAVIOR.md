# Desktop composer behavior

## Feature contract

| Input | Required custom behavior |
|---|---|
| Ctrl+A | Select the complete editable prompt. It must not merely move the cursor. |
| Home / End | Move to the start / end of the current prompt line. |
| Shift+Home / Shift+End | Select to the start / end of the current prompt line. |
| Ctrl+Home / Ctrl+End | Move to the start / end of the complete prompt. |
| Ctrl+Shift+Home / Ctrl+Shift+End | Select to the start / end of the complete prompt. |
| `Shift+Left` / `Shift+Right` | Extend or shrink the prompt selection by a grapheme. |
| `Shift+Up` / `Shift+Down` | Extend the selection across visual or logical lines. |
| `Ctrl+Shift+Left` / `Ctrl+Shift+Right` | Extend the selection by word boundaries. |
| Left-button click and drag | Select editable composer text, including multiple lines. |
| Double-click / triple-click | Select a word / line. |
| Typing with a selection | Replace the selected prompt text. |
| `Backspace` or `Delete` with a selection | Delete the entire selected prompt range. |
| `Ctrl+C` with a selection | Copy selected prompt text to the Windows clipboard. |
| `Ctrl+C` without a selection | Preserve Codex interrupt/cancel behavior. |
| `Ctrl+X` | Copy and remove selected prompt text. |
| `Ctrl+V` | Paste non-empty clipboard text first; otherwise attach a clipboard image. |
| `Ctrl+Alt+V` | Force clipboard-image paste. |
| `Ctrl+Z` | Undo the last composer edit. |
| `Ctrl+Shift+Z` | Redo the last undone composer edit. `Ctrl+Y` remains the upstream yank binding. |

Navigation and selections inside the editable prompt belong to the Codex composer rather than VS Code's rendered
terminal buffer. The bundled Terminal Bridge forwards `Ctrl+Home` and `Ctrl+End` only while the active CLI Editor composer owns input. When capture is released or no matching live CLI Editor process exists, it invokes VS Code's native terminal-top or terminal-bottom command instead. Typing and deletion can replace a mouse-dragged prompt selection.
Terminal mouse capture is enabled while composer interaction is needed and restored on exit.

When a wheel or two-finger touchpad gesture begins in the normal chat view, Codex releases mouse
capture so VS Code can scroll its native terminal history. The first wheel event performs the
handoff; continue the same gesture to scroll. The next keyboard or paste event restores mouse
capture for composer click/drag editing.

When an assistant turn finishes in the normal chat view and the composer is empty, Codex also
releases mouse capture. This lets VS Code select and copy completed transcript text immediately,
without requiring a wheel event first. If the composer contains a draft or an alternate-screen
overlay is active, capture stays enabled so click placement and drag selection keep working. With
an empty composer, press `Right Arrow` before clicking if you want to restore capture without
changing the prompt.

## Clipboard rule

`Ctrl+V` reads a single clipboard handle and applies this priority:

1. Non-empty text.
2. Image data or an image file from the clipboard.
3. A clear unsupported-content error.

This prevents copied text from being misclassified as an image and producing `Failed to paste
image`. Large text pastes continue through Codex's existing large-paste placeholder path.

## Scope

The prompt-editing behavior applies to the custom Codex CLI, including its pre-session startup draft. The signed Windows bundle includes a small VS Code extension that contributes two terminal keybindings but does not rewrite user settings. It defaults safely to native VS Code scrolling for Claude Code, ordinary terminals, stale or malformed bridge state, and any process that is not the active CLI Editor Codex session. The launcher does not patch Claude Code, VS Code, or the npm Codex package.
