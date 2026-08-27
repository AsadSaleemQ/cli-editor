use std::path::Path;

use ed25519_dalek::{Signer, SigningKey};
use zeroize::{Zeroize, Zeroizing};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if arguments.len() != 5 {
        return Err("usage: sign_release PRIVATE_KEY MANIFEST SIGNATURE_OUT PUBLIC_KEY_OUT".into());
    }
    let private_key_path = Path::new(&arguments[1]);
    let manifest_path = Path::new(&arguments[2]);
    let signature_path = Path::new(&arguments[3]);
    let public_key_path = Path::new(&arguments[4]);
    let private_key = Zeroizing::new(if private_key_path == Path::new("-") {
        std::env::var("CLI_EDITOR_SIGNING_PRIVATE_KEY_HEX")?
    } else {
        std::fs::read_to_string(private_key_path)?
    });
    let decoded = Zeroizing::new(hex::decode(private_key.trim())?);
    let mut seed: [u8; 32] = decoded
        .as_slice()
        .try_into()
        .map_err(|_| "private key must be exactly 32 bytes encoded as hexadecimal")?;
    let key = SigningKey::from_bytes(&seed);
    seed.zeroize();
    let manifest = std::fs::read(manifest_path)?;
    std::fs::write(
        signature_path,
        format!("{}\n", hex::encode(key.sign(&manifest).to_bytes())),
    )?;
    std::fs::write(
        public_key_path,
        format!("{}\n", hex::encode(key.verifying_key().to_bytes())),
    )?;
    Ok(())
}
