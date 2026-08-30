# Update and rollback contract

CLI Editor owns `%LOCALAPPDATA%\CLIEditor` and one per-user PATH entry pointing to its `shims` directory. It never edits native Codex, npm packages, or shell profile files in place. When VS Code is discovered, install adds the signed bundled `asadsaleemq.cli-editor` extension and records whether CLI Editor added it. VS Code profiles have independent extension and keybinding registries; profile-specific bindings remain user-owned configuration.

## Native CLI updates

Each launch checks the recorded canonical executable path, package root, expected package family, file size, and modification time. A metadata change triggers a bounded version/hash reinspection of that exact path. A legitimate in-place update is atomically adopted and journaled. Runtime discovery never searches PATH to replace a changed target. Relocation, package-family changes, unsafe extensions, and unrecognized script launchers fail closed until `repair --adopt-native` is explicitly run. Official npm shims are resolved only through exact package metadata to their native executable.

A native Codex update never authorizes an enhanced binary. The signed manifest must independently support the exact native Codex version. Until a signed release lists it, defaulted Codex launches the verified native CLI and an explicit `codex cli-editor` request fails visibly. An unlisted VS Code host version warns and continues because host drift does not change the pinned Codex binary.

## CLI Editor updates

Users download a release bundle explicitly and run:

```powershell
cli-editor update --bundle C:\path\to\extracted-release
```

The updater verifies the Ed25519 signature, monotonic sequence, issue/expiry window, minimum dispatcher version, Codex-only compatibility metadata, and each artifact's exact size and SHA-256. Enhanced Codex and newly discovered or rewritten native Codex use a 60-second cold-artifact probe budget. After an in-place native update, a probe timeout forces warning-only native routing for default Codex once path, package root, identity family, and executable shape are revalidated; an explicit enhanced Codex request still fails visibly rather than silently changing mode. Activation also removes legacy Claude routing state and an old `claude.exe` shim without changing the separately installed native CLI.

Activation runs under the cross-process state lock. It publishes a new immutable version directory, signed manifest cache, optional dispatcher shims, and state record as one recoverable operation. File replacements are backed up and restored if the closure or atomic state save fails. The active release plus two prior signed releases are retained, and active sessions continue using already-open binaries.

A dispatcher-changing update must be launched from the new external bundle. This avoids replacing the running Windows executable. In-use shims cause a clean update failure and restoration rather than a mixed version.

## Rollback

```powershell
cli-editor rollback
cli-editor rollback --release RELEASE_DIRECTORY_NAME
```

Without `--release`, CLI Editor selects the newest retained valid release older than the active manifest sequence. The named form selects one immutable directory under the owned `versions` directory. Before activation it re-verifies the retained manifest signature, expiry, artifact hashes and sizes, dispatcher minimum, and enhanced Codex smoke version. Rollback updates the active manifest cache but preserves the highest sequence ever observed, so an old bundle cannot later masquerade as a new update. Rollback intentionally keeps the newest dispatcher shims in place and switches only the verified enhanced payload, active manifest cache, and state. If publication fails, the previous manifest cache and state are restored.

## Compatibility fallback

A defaulted enhanced Codex route falls back to verified native Codex when the manifest is invalid, expired beyond grace, incompatible, or the enhanced artifact fails validation. An explicit `codex cli-editor` request reports the problem and exits rather than silently changing the requested mode.

## Uninstall

`cli-editor uninstall` restores the exact raw pre-install user PATH registry value and type when it is still the value CLI Editor installed. If PATH changed afterward, it removes only CLI Editor's entry and preserves those later edits. When the owned shim entry already existed before installation, CLI Editor did not add or own that user setting; uninstall deliberately preserves it, prints a notice, and removes the owned directory, so the retained entry may then point to a missing directory. It broadcasts the environment change, removes the companion VS Code extension only when CLI Editor originally added it, removes owned shims/state/releases, and leaves native Codex and pre-existing extensions untouched. When uninstall runs through its own installed shim, it first removes that executable from the command path, continues state and PATH cleanup even if Windows keeps the running image locked, and attempts to schedule the renamed residue for deletion at restart. If non-elevated Windows cannot queue that deletion, CLI Editor reports the exact inert file; it can be removed after the command exits. Terminals that were already open still hold their old PATH snapshot, so start a new terminal before using Codex again.
