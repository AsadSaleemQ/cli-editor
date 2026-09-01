use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::error::{CodexCliEditorError, Result};

pub const MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const EXPIRY_GRACE_SECONDS: u64 = 30 * 24 * 60 * 60;
const DEVELOPMENT_PUBLIC_KEY_HEX: &str =
    "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
const CONFIGURED_PUBLIC_KEY_HEX: &str = include_str!("../compatibility/public-key.hex");
#[cfg(not(test))]
const VERIFICATION_PUBLIC_KEY_HEX: &str = CONFIGURED_PUBLIC_KEY_HEX;
#[cfg(test)]
const VERIFICATION_PUBLIC_KEY_HEX: &str = DEVELOPMENT_PUBLIC_KEY_HEX;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityManifest {
    pub schema_version: u32,
    pub sequence: u64,
    pub issued_unix: u64,
    pub expires_unix: u64,
    pub minimum_dispatcher_version: String,
    pub compatibility: Vec<CompatibilityEntry>,
    pub artifacts: Vec<Artifact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityEntry {
    pub codex: String,
    pub vscode: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Freshness {
    Fresh,
    Grace { stale_seconds: u64 },
    Expired,
}

#[derive(Debug)]
pub struct VerifiedManifest {
    pub manifest: CompatibilityManifest,
    pub freshness: Freshness,
}

pub fn verify_manifest(
    bytes: &[u8],
    signature_hex: &str,
    highest_accepted_sequence: u64,
    now_unix: u64,
) -> Result<VerifiedManifest> {
    verify_manifest_with_key_hex(
        bytes,
        signature_hex,
        highest_accepted_sequence,
        now_unix,
        VERIFICATION_PUBLIC_KEY_HEX,
    )
}

fn verify_manifest_with_key_hex(
    bytes: &[u8],
    signature_hex: &str,
    highest_accepted_sequence: u64,
    now_unix: u64,
    public_key_hex: &str,
) -> Result<VerifiedManifest> {
    let key_bytes: [u8; 32] = hex::decode(public_key_hex.trim())
        .map_err(|_| CodexCliEditorError::InvalidManifestKey)?
        .try_into()
        .map_err(|_| CodexCliEditorError::InvalidManifestKey)?;
    let signature_bytes: [u8; 64] = hex::decode(signature_hex.trim())
        .map_err(|_| CodexCliEditorError::InvalidManifestSignature)?
        .try_into()
        .map_err(|_| CodexCliEditorError::InvalidManifestSignature)?;
    let key = VerifyingKey::from_bytes(&key_bytes)
        .map_err(|_| CodexCliEditorError::InvalidManifestKey)?;
    let signature = Signature::from_bytes(&signature_bytes);
    key.verify(bytes, &signature)
        .map_err(|_| CodexCliEditorError::InvalidManifestSignature)?;

    let manifest: CompatibilityManifest = serde_json::from_slice(bytes)?;
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(CodexCliEditorError::UnsupportedManifestSchema(
            manifest.schema_version,
        ));
    }
    if manifest.sequence < highest_accepted_sequence {
        return Err(CodexCliEditorError::ManifestRollback {
            highest: highest_accepted_sequence,
            received: manifest.sequence,
        });
    }
    if manifest.issued_unix > manifest.expires_unix {
        return Err(CodexCliEditorError::InvalidManifestWindow);
    }
    let freshness = if now_unix <= manifest.expires_unix {
        Freshness::Fresh
    } else {
        let stale_seconds = now_unix - manifest.expires_unix;
        if stale_seconds <= EXPIRY_GRACE_SECONDS {
            Freshness::Grace { stale_seconds }
        } else {
            Freshness::Expired
        }
    };
    Ok(VerifiedManifest {
        manifest,
        freshness,
    })
}

pub fn release_key_is_development() -> bool {
    CONFIGURED_PUBLIC_KEY_HEX.trim() == DEVELOPMENT_PUBLIC_KEY_HEX
}

impl CompatibilityManifest {
    pub fn supports_codex(&self, codex: &str) -> bool {
        self.compatibility.iter().any(|entry| entry.codex == codex)
    }

    pub fn artifact(&self, name: &str) -> Option<&Artifact> {
        self.artifacts.iter().find(|artifact| artifact.name == name)
    }
    pub fn supports(&self, codex: &str, vscode: &str) -> bool {
        self.compatibility.iter().any(|entry| {
            entry.codex == codex && entry.vscode.iter().any(|version| version == vscode)
        })
    }
}

#[cfg(test)]
fn verify_with_key(
    bytes: &[u8],
    signature_bytes: &[u8; 64],
    key_bytes: &[u8; 32],
    highest: u64,
    now: u64,
) -> Result<VerifiedManifest> {
    let key =
        VerifyingKey::from_bytes(key_bytes).map_err(|_| CodexCliEditorError::InvalidManifestKey)?;
    key.verify(bytes, &Signature::from_bytes(signature_bytes))
        .map_err(|_| CodexCliEditorError::InvalidManifestSignature)?;
    let manifest: CompatibilityManifest = serde_json::from_slice(bytes)?;
    if manifest.sequence < highest {
        return Err(CodexCliEditorError::ManifestRollback {
            highest,
            received: manifest.sequence,
        });
    }
    let stale = now.saturating_sub(manifest.expires_unix);
    let freshness = if stale == 0 {
        Freshness::Fresh
    } else if stale <= EXPIRY_GRACE_SECONDS {
        Freshness::Grace {
            stale_seconds: stale,
        }
    } else {
        Freshness::Expired
    };
    Ok(VerifiedManifest {
        manifest,
        freshness,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn production_key_verifies_the_codex_release_fixture() {
        let bytes = include_bytes!("../compatibility/production-manifest-fixture.json");
        let signature = include_str!("../compatibility/production-manifest-fixture.sig");
        let verified = super::verify_manifest_with_key_hex(
            bytes,
            signature,
            41,
            1_800_000_000,
            super::CONFIGURED_PUBLIC_KEY_HEX,
        )
        .unwrap();
        assert_eq!(verified.manifest.sequence, 42);
        assert!(verified.manifest.supports_codex("0.148.0"));
    }

    #[test]
    fn configured_release_key_is_valid_and_not_the_development_key() {
        let configured = hex::decode(super::CONFIGURED_PUBLIC_KEY_HEX.trim()).unwrap();
        assert_eq!(configured.len(), 32);
        assert_ne!(
            super::CONFIGURED_PUBLIC_KEY_HEX.trim(),
            super::DEVELOPMENT_PUBLIC_KEY_HEX
        );
        assert!(!super::release_key_is_development());
    }

    use ed25519_dalek::{Signer, SigningKey};

    use super::{Freshness, verify_with_key};

    #[test]
    fn signed_manifest_enforces_sequence_and_grace() {
        let key = SigningKey::from_bytes(&[7u8; 32]);
        let bytes = br#"{"schema_version":1,"sequence":9,"issued_unix":100,"expires_unix":200,"minimum_dispatcher_version":"0.1.0","compatibility":[],"artifacts":[]}"#;
        let signature = key.sign(bytes);
        let verified = verify_with_key(
            bytes,
            &signature.to_bytes(),
            &key.verifying_key().to_bytes(),
            8,
            201,
        )
        .unwrap();
        assert_eq!(verified.manifest.sequence, 9);
        assert_eq!(verified.freshness, Freshness::Grace { stale_seconds: 1 });
        assert!(
            verify_with_key(
                bytes,
                &signature.to_bytes(),
                &key.verifying_key().to_bytes(),
                10,
                150
            )
            .is_err()
        );
    }
}
