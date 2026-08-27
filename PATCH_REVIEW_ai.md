# Patch review strategy

The distributable patch is intentionally atomic: mouse capture, event routing, text selection, clipboard behavior, and composer state share types and must compile together. For review, split it into two logical passes without applying partial states.

## Pass A: terminal and event boundary

Review `tui.rs`, `tui/event_stream.rs`, `tui/screen_size.rs`, `tui/windows_console.rs`, `app.rs`, prompt/picker screens, and chat-widget routing. Confirm:

- mouse capture is armed for interactive composer editing, including inline mode, released during scrollback handoff or after a completed turn with an empty composer, retained for a non-empty draft, restored by subsequent composer input, and disabled on exit;
- scroll/click/drag events reach the composer without leaking escape sequences;
- prompt screens explicitly consume or route the new mouse event;
- Ctrl+C, Ctrl+Break, close/logoff/shutdown, resize, suspend/resume, and paste paths remain intact;
- native key mappings such as Ctrl+Y yank and Shift+Up/Down reasoning shortcuts are preserved.

## Pass B: composer model and rendering

Review `bottom_pane/chat_composer.rs`, `bottom_pane/textarea.rs`, `bottom_pane/textarea/selection.rs`, clipboard paste, footer hints, snapshots, and composer tests. Confirm:

- selection uses character boundaries and reverse-video rendering;
- click placement and drag selection handle wrapped Unicode text;
- Ctrl+A/C/X/V and image paste preserve existing behavior;
- undo/redo does not shadow upstream yank;
- touchpad/wheel scrolling changes viewport state without corrupting input;
- selection is cleared or retained consistently across edit operations.

## Atomic application evidence

Pinned upstream: `rust-v0.148.0` at `3ba0f711642a888aec92a611a3f3b2211157ff89`.

Current patch: 35 files, 94,244 bytes, SHA-256 `6a87f803bd7f6175c47419ecb0f254c0f62063239ce6007a01c670b259cff08c`.

The patch passes `git diff --check` and reverses cleanly against the current pinned worktree. A clean-tree forward-application check is a release gate. Partial patch application is not supported.
## Upstream bump checklist

Every Codex or crossterm version bump requires a fresh patch application, compile, and full-suite comparison. In particular, review `crossterm::event::Event` exhaustiveness: the patch deliberately matches known variants without a wildcard so a new upstream event fails at compile time instead of being silently discarded.

Keep mouse-capture disposition single-sourced through prepare_mouse_capture; composer and non-composer entry points may select the alt-screen policy but must not duplicate the event-kind handling.
