# CLI Editor

CLI Editor is a Windows launcher and maintained Codex patch that adds desktop-style terminal editing: touchpad and wheel scrolling, mouse placement and drag selection, clipboard shortcuts, image paste, undo/redo, and full-prompt Ctrl+Home/End navigation. Claude Code is never patched or redistributed; its route is a validated native pass-through.

## Install and use

Download and extract a signed Windows release, then run:

```powershell
.\cli-editor.exe install
```

Open a new terminal so it inherits the updated user PATH, then run:

```powershell
codex cli-editor
claude cli-editor
```

Make enhanced Codex or managed native Claude the default with `cli-editor default codex`, `cli-editor default claude`, or `cli-editor default all`. Restore native defaults with `cli-editor restore all`. Remove CLI Editor with `cli-editor uninstall`; it restores the exact pre-install user PATH when it owned the change, preserves later edits, and leaves any pre-existing unowned shim entry in place with a notice. If Windows keeps the running shim image locked, the command still completes settings removal, removes that shim from command resolution, and reports any inert residue that can be deleted after exit.

Mouse editing requires terminal mouse capture in the interactive composer, including inline mode. A wheel/touchpad gesture temporarily releases capture so VS Code receives native scrollback; finishing an assistant turn with an empty composer in the normal chat view also releases capture so completed transcript text can be selected immediately. Keyboard or paste restores capture, and a non-empty draft keeps capture armed for composer placement and drag selection.

Current release baseline: Windows 11 x64, VS Code 1.134.0, Codex CLI 0.148.0, and native Claude Code 2.1.240. Codex compatibility remains exact and fail-closed for explicit enhanced requests; an unknown VS Code host version produces a visible warning and continues because the host does not change the pinned Codex binary. Suspicious native target changes fail closed. The release workflow signs, hashes, SBOM-attests, provenance-attests, and independently rebuilds assets; it refuses publication unless all artifact hashes match bit-for-bit. v0.1 is not Authenticode-signed, so Windows SmartScreen may warn on first run.

Use `codex -- cli-editor` or `claude -- cli-editor` when `cli-editor` must be passed literally as the native CLI's first argument. See [TECHNICAL_GUIDE.md](TECHNICAL_GUIDE.md) for architecture, trust, build, update, compatibility, and testing details.

## Project boundary

CLI Editor is an independent modified build and is not affiliated with or endorsed by OpenAI, Anthropic, or Microsoft. Codex source is used under Apache-2.0; Claude Code is neither modified nor redistributed.
