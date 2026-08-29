# CLI Editor

CLI Editor is a Windows launcher and maintained Codex patch that adds desktop-style terminal editing: touchpad and wheel scrolling, mouse placement and drag selection, clipboard shortcuts, image paste, undo/redo, and editable-prompt Ctrl+Home/End navigation. Claude Code is never patched or redistributed; its route is a validated native pass-through.

## Install and use

Download the latest published Windows release from a terminal with GitHub CLI:

```powershell
gh release download --repo AsadSaleemQ/cli-editor --pattern 'cli-editor-*-windows-x64.zip' --pattern 'cli-editor-*-windows-x64.zip.sha256'
```

Verify the adjacent SHA-256 file, extract the ZIP, then run:

```powershell
.\cli-editor.exe install
```

Installation also installs the bundled CLI Editor Terminal Bridge when VS Code is detected. Reload VS Code once so Ctrl+Home/End can be routed conditionally without changing your user keybindings.

To download the lean source repository for development instead:

```powershell
git clone https://github.com/AsadSaleemQ/cli-editor.git
cd cli-editor
```

The source repository intentionally does not contain compiled release executables, so cloning it is
not an installation method.

Open a new terminal so it inherits the updated user PATH, then run:

```powershell
codex cli-editor
claude cli-editor
```

Make enhanced Codex or managed native Claude the default with `cli-editor default codex`, `cli-editor default claude`, or `cli-editor default all`. Restore native defaults with `cli-editor restore all`. Remove CLI Editor with `cli-editor uninstall`; it restores the exact pre-install user PATH when it owned the change, preserves later edits, and leaves any pre-existing unowned shim entry in place with a notice. If Windows keeps the running shim image locked, the command still completes settings removal, removes that shim from command resolution, and reports any inert residue that can be deleted after exit.

Mouse editing requires terminal mouse capture in the interactive composer, including inline mode. A wheel/touchpad gesture temporarily releases capture so VS Code receives native scrollback; finishing an assistant turn with an empty composer in the normal chat view also releases capture so completed transcript text can be selected immediately. Keyboard or paste restores capture, and a non-empty draft keeps capture armed for composer placement and drag selection.

In VS Code, Ctrl+Home and Ctrl+End move to the beginning and end of the editable prompt for the lifetime of the active enhanced Codex session, including after wheel or completed-turn scrollback handoff. Other terminals and native Claude retain their host defaults. Ctrl+Shift prompt-boundary selection remains available in terminals that forward those key sequences.

Current release baseline: Windows 11 x64, VS Code 1.134.0 and 1.135.0, Codex CLI 0.148.0, and native Claude Code 2.1.240 and 2.1.251. Codex compatibility remains exact and fail-closed for explicit enhanced requests; an unknown VS Code host version produces a visible warning and continues because the host does not change the pinned Codex binary. Suspicious native target changes fail closed. The hosted release workflow hashes, provenance-attests, and independently rebuilds unsigned assets, refusing a candidate unless all artifact hashes match bit-for-bit. Signing happens only on the maintainer workstation after a successful hosted run; the private seed is never uploaded. v0.1 is not Authenticode-signed, so Windows SmartScreen may warn on first run.

Use `codex -- cli-editor` or `claude -- cli-editor` when `cli-editor` must be passed literally as the native CLI's first argument. See [TECHNICAL_GUIDE.md](TECHNICAL_GUIDE.md) for architecture, trust, build, update, compatibility, and testing details.

## Project boundary

CLI Editor is an independent modified build and is not affiliated with or endorsed by OpenAI, Anthropic, or Microsoft. Codex source is used under Apache-2.0; Claude Code is neither modified nor redistributed.
