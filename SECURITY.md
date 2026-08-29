# Security policy

## Supported versions

Only the newest published CLI Editor release is supported. Compatibility with Codex, Claude Code, and VS Code is narrower than binary support and is declared by the signed compatibility manifest.

## Report a vulnerability

Do not open a public issue for a suspected vulnerability. Use GitHub's private vulnerability reporting for `AsadSaleemQ/cli-editor`. Include the affected version, reproduction steps, expected impact, and whether the issue requires an attacker already running code as the same Windows user.

## Trust model

The release manifest signs the companion VSIX as an artifact. Its two terminal commands send fixed Ctrl+Home and Ctrl+End escape sequences to the active terminal; they do not read, store, or transmit prompt or transcript content. VS Code user or profile keybindings determine whether those commands receive the corresponding chords.

CLI Editor protects against release-asset substitution, network/CDN tampering, cached-manifest corruption, sequence rollback, unsafe PATH discovery, shim recursion, and incompatible enhanced activation. It does not claim to protect against arbitrary code already executing as the same Windows user.

Native target discovery accepts canonical `.exe` paths or resolves official npm shims through exact package names and fixed native-binary locations. CLI Editor never executes a discovered `.cmd`, `.bat`, or `.ps1` wrapper; the first matching unknown package or arbitrary script is rejected so discovery cannot silently change PATH semantics.

The Ed25519 public key is embedded in the dispatcher. Release manifests are verified on every use. The private seed is kept outside Git, GitHub, CI, and release artifacts. Workflow-dispatch values enter scripts only through environment variables and must pass version, expiry, package-version, and repository-wide manifest-sequence validation. Release runs are serialized, and each requested sequence must exceed the maximum found in existing release and draft manifest assets after the prior run finishes. Exact artifact allowlists reject unexpected bundle or top-level files. Independent unsigned builds must match bit-for-bit before GitHub attests their provenance. Only then may the maintainer download the candidate and run the local finalizer, which checks that the run succeeded at the exact local commit, verifies the manifest inventory, signs locally, uploads a draft, clears the key environment variable, and removes the temporary artifact workspace. GitHub jobs never receive the seed. The signer zeroizes the seed string, decoded buffers, stack copy, and signing-key material. A compromised signing key requires an out-of-band dispatcher release and public advisory; the compromised key cannot authorize its own replacement.

## Update behavior

Startup never performs a blocking network download. Users explicitly download a release bundle and run `cli-editor update --bundle DIRECTORY`. Signature, sequence, expiry, size, hash, smoke-test, lock, or state-publication failure leaves the prior active release in place.
